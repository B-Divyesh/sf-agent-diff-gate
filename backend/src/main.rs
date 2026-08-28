use axum::{
    extract::{Path, State},
    http::{HeaderName, HeaderValue, StatusCode},
    middleware,
    response::{IntoResponse, Response},
    routing::{get, get_service},
    Json, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePoolOptions, FromRow, SqlitePool};
use std::{
    collections::HashMap,
    env,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tower_http::{
    services::{ServeDir, ServeFile},
    set_header::SetResponseHeaderLayer,
    trace::TraceLayer,
};
use tracing::info;
use uuid::Uuid;

const RATE_LIMIT_PER_SECOND: u32 = 40;
const RETRY_AFTER_SECONDS: &str = "1";

#[derive(Clone)]
struct AppState {
    db: SqlitePool,
    build: String,
    limits: Arc<Mutex<HashMap<String, Window>>>,
}

struct Window {
    since: Instant,
    hits: u32,
}

#[derive(Serialize)]
struct Health {
    status: &'static str,
    build: String,
}

#[derive(Serialize, FromRow)]
struct Packet {
    id: String,
    title: String,
    owner: String,
    status: String,
    data: String,
    created_at: String,
}

#[derive(Deserialize)]
struct NewPacket {
    title: String,
    owner: String,
    data: serde_json::Value,
}

async fn health(State(state): State<AppState>) -> Json<Health> {
    Json(Health {
        status: "ok",
        build: state.build,
    })
}

async fn list_packets(State(state): State<AppState>) -> Result<Json<Vec<Packet>>, AppError> {
    Ok(Json(
        sqlx::query_as(
            "SELECT id,title,owner,status,data,created_at FROM packets ORDER BY created_at DESC LIMIT 50",
        )
        .fetch_all(&state.db)
        .await?,
    ))
}

async fn create_packet(
    State(state): State<AppState>,
    Json(input): Json<NewPacket>,
) -> Result<(StatusCode, Json<Packet>), AppError> {
    if input.title.trim().is_empty() || input.title.len() > 180 || input.owner.trim().is_empty() {
        return Err(AppError::bad("Add a title and an owner before saving."));
    }

    let packet = Packet {
        id: Uuid::new_v4().to_string(),
        title: input.title.trim().to_string(),
        owner: input.owner.trim().to_string(),
        status: "needs review".into(),
        data: input.data.to_string(),
        created_at: Utc::now().to_rfc3339(),
    };
    sqlx::query("INSERT INTO packets (id,title,owner,status,data,created_at) VALUES (?,?,?,?,?,?)")
        .bind(&packet.id)
        .bind(&packet.title)
        .bind(&packet.owner)
        .bind(&packet.status)
        .bind(&packet.data)
        .bind(&packet.created_at)
        .execute(&state.db)
        .await?;
    Ok((StatusCode::CREATED, Json(packet)))
}

async fn get_packet(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Packet>, AppError> {
    sqlx::query_as("SELECT id,title,owner,status,data,created_at FROM packets WHERE id=?")
        .bind(id)
        .fetch_optional(&state.db)
        .await?
        .map(Json)
        .ok_or_else(|| AppError::not_found("That review packet was not found."))
}

fn client_ip(headers: &axum::http::HeaderMap) -> String {
    headers
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("local")
        .to_string()
}

async fn rate_limit(
    State(state): State<AppState>,
    request: axum::extract::Request,
    next: middleware::Next,
) -> Response {
    let ip = client_ip(request.headers());
    let blocked = {
        let mut limits = state.limits.lock().expect("rate limiter mutex");
        let window = limits.entry(ip).or_insert(Window {
            since: Instant::now(),
            hits: 0,
        });
        if window.since.elapsed() > Duration::from_secs(1) {
            window.since = Instant::now();
            window.hits = 0;
        }
        window.hits += 1;
        window.hits > RATE_LIMIT_PER_SECOND
    };

    if blocked {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            [("retry-after", RETRY_AFTER_SECONDS)],
            "Too many requests. Try again in one second.",
        )
            .into_response();
    }

    next.run(request).await
}

#[derive(Debug)]
struct AppError {
    status: StatusCode,
    message: String,
}

impl AppError {
    fn bad(message: &str) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
        }
    }

    fn not_found(message: &str) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: message.into(),
        }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(_: sqlx::Error) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: "The packet store is unavailable. Try again shortly.".into(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(serde_json::json!({ "error": self.message })),
        )
            .into_response()
    }
}

fn static_routes() -> Router<AppState> {
    let index = || get_service(ServeFile::new("dist/index.html"));
    Router::new()
        .route("/", index())
        .route("/demo", index())
        .route("/privacy", index())
        .route("/terms", index())
        .route("/404", index())
        .nest_service("/assets", ServeDir::new("dist/assets"))
        .route_service("/favicon.svg", ServeFile::new("dist/favicon.svg"))
        .route_service(
            "/apple-touch-icon.png",
            ServeFile::new("dist/apple-touch-icon.png"),
        )
        .route_service(
            "/change-control.webp",
            ServeFile::new("dist/change-control.webp"),
        )
        .route_service("/social.webp", ServeFile::new("dist/social.webp"))
        .route_service("/robots.txt", ServeFile::new("dist/robots.txt"))
        .route_service("/sitemap.xml", ServeFile::new("dist/sitemap.xml"))
        .route_service(
            "/manifest.webmanifest",
            ServeFile::new("dist/manifest.webmanifest"),
        )
        .fallback_service(ServeFile::new("dist/index.html"))
}

fn app(state: AppState) -> Router {
    let protected = Router::new()
        .route("/api/packets", get(list_packets).post(create_packet))
        .route("/api/packets/:id", get(get_packet))
        .merge(static_routes())
        .layer(middleware::from_fn_with_state(state.clone(), rate_limit));

    Router::new()
        .route("/health", get(health))
        .merge(protected)
        .layer(TraceLayer::new_for_http())
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("x-content-type-options"),
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("referrer-policy"),
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("content-security-policy"),
            HeaderValue::from_static(
                "default-src 'self'; img-src 'self' data:; style-src 'self'; script-src 'self'; connect-src 'self' https://api.sociobot.in",
            ),
        ))
        .with_state(state)
}

async fn create_schema(db: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query("CREATE TABLE IF NOT EXISTS packets (id TEXT PRIMARY KEY, title TEXT NOT NULL, owner TEXT NOT NULL, status TEXT NOT NULL, data TEXT NOT NULL, created_at TEXT NOT NULL)")
        .execute(db)
        .await?;
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter("info")
        .init();

    let port = env::var("PORT").unwrap_or_else(|_| "8080".into());
    let build = env::var("BUILD_SHA").unwrap_or_else(|_| "dev".into());
    let (db_url, database_config) = match env::var("DATABASE_URL") {
        Ok(value) => (value, "supplied"),
        Err(_) => (
            "sqlite:/data/diff-gate.db?mode=rwc".into(),
            "generated default",
        ),
    };
    if !std::path::Path::new("/data").exists() {
        std::fs::create_dir_all("/data").ok();
    }
    info!(
        database_config,
        "Diff Gate starting with no required runtime configuration"
    );

    let db = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;
    create_schema(&db).await?;
    let state = AppState {
        db,
        build,
        limits: Arc::new(Mutex::new(HashMap::new())),
    };
    let address = SocketAddr::from(([0, 0, 0, 0], port.parse()?));
    info!(%address, "listening");
    let listener = tokio::net::TcpListener::bind(address).await?;
    axum::serve(listener, app(state))
        .with_graceful_shutdown(shutdown())
        .await?;
    Ok(())
}

async fn shutdown() {
    let _ = tokio::signal::ctrl_c().await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::to_bytes, http::Request};
    use tower::ServiceExt;

    async fn test_app() -> Router {
        let db = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("in-memory database");
        create_schema(&db).await.expect("packet schema");
        app(AppState {
            db,
            build: "test-sha".into(),
            limits: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    #[tokio::test]
    async fn health_reports_the_build_identity() {
        let response = test_app()
            .await
            .oneshot(
                Request::get("/health")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(body.as_ref(), br#"{"status":"ok","build":"test-sha"}"#);
    }

    #[tokio::test]
    async fn rate_limit_uses_the_first_forwarded_ip_and_returns_retry_after() {
        let app = test_app().await;
        for _ in 0..RATE_LIMIT_PER_SECOND {
            let response = app
                .clone()
                .oneshot(
                    Request::get("/api/packets")
                        .header("x-forwarded-for", "198.51.100.7, 10.0.0.5")
                        .body(axum::body::Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);
        }
        let response = app
            .oneshot(
                Request::get("/api/packets")
                    .header("x-forwarded-for", "198.51.100.7, 10.0.0.5")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(response.headers()["retry-after"], RETRY_AFTER_SECONDS);
    }

    #[test]
    fn docker_build_stage_stays_compatible_with_locked_icu_2_3() {
        let dockerfile = include_str!("../../Dockerfile");
        let lockfile = include_str!("../../Cargo.lock");
        assert!(lockfile.contains("name = \"icu_collections\"\nversion = \"2.3.0\""));
        assert!(
            dockerfile.contains("FROM rust:1.88-alpine AS build"),
            "ICU 2.3 requires rustc 1.88 or newer; keep the Docker build stage at Rust 1.88+"
        );
        assert!(dockerfile.contains("COPY Cargo.toml Cargo.lock ./"));
        assert!(dockerfile.contains("EXPOSE 8080"));
    }
}
