use axum::{extract::{Path, State}, http::{HeaderName, HeaderValue, StatusCode}, middleware, response::{IntoResponse, Response}, routing::{get, get_service}, Json, Router};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePoolOptions, FromRow, SqlitePool};
use std::{collections::HashMap, env, net::SocketAddr, sync::{Mutex, OnceLock}, time::{Duration, Instant}};
use tower_http::{services::{ServeDir, ServeFile}, set_header::SetResponseHeaderLayer, trace::TraceLayer};
use tracing::info;
use uuid::Uuid;

#[derive(Clone)] struct AppState { db: SqlitePool, build: String }
struct Window { since: Instant, hits: u32 }
static LIMITS: OnceLock<Mutex<HashMap<String, Window>>> = OnceLock::new();
#[derive(Serialize)] struct Health { status: &'static str, build: String }
#[derive(Serialize, FromRow)] struct Packet { id:String, title:String, owner:String, status:String, data:String, created_at:String }
#[derive(Deserialize)] struct NewPacket { title:String, owner:String, data:serde_json::Value }

async fn health(State(s): State<AppState>) -> Json<Health> { Json(Health { status:"ok", build:s.build }) }
async fn list_packets(State(s): State<AppState>) -> Result<Json<Vec<Packet>>, AppError> { Ok(Json(sqlx::query_as("SELECT id,title,owner,status,data,created_at FROM packets ORDER BY created_at DESC LIMIT 50").fetch_all(&s.db).await?)) }
async fn create_packet(State(s): State<AppState>, Json(input): Json<NewPacket>) -> Result<(StatusCode, Json<Packet>), AppError> {
    if input.title.trim().is_empty() || input.title.len()>180 || input.owner.trim().is_empty() { return Err(AppError::bad("Add a title and an owner before saving.")); }
    let packet=Packet{id:Uuid::new_v4().to_string(), title:input.title.trim().to_string(), owner:input.owner.trim().to_string(),status:"needs review".into(),data:input.data.to_string(),created_at:Utc::now().to_rfc3339()};
    sqlx::query("INSERT INTO packets (id,title,owner,status,data,created_at) VALUES (?,?,?,?,?,?)").bind(&packet.id).bind(&packet.title).bind(&packet.owner).bind(&packet.status).bind(&packet.data).bind(&packet.created_at).execute(&s.db).await?;
    Ok((StatusCode::CREATED, Json(packet)))
}
async fn get_packet(State(s): State<AppState>, Path(id): Path<String>) -> Result<Json<Packet>, AppError> { sqlx::query_as("SELECT id,title,owner,status,data,created_at FROM packets WHERE id=?").bind(id).fetch_optional(&s.db).await?.map(Json).ok_or_else(|| AppError::not_found("That review packet was not found.")) }
async fn rate_limit(req: axum::extract::Request, next: middleware::Next) -> Response {
    let ip=req.headers().get("x-forwarded-for").and_then(|v|v.to_str().ok()).and_then(|v|v.split(',').next()).unwrap_or("local").trim().to_string();
    let blocked = { let mut map=LIMITS.get_or_init(||Mutex::new(HashMap::new())).lock().expect("rate limiter"); let window=map.entry(ip).or_insert(Window{since:Instant::now(),hits:0}); if window.since.elapsed()>Duration::from_secs(1) { window.since=Instant::now(); window.hits=0; } window.hits+=1; window.hits>40 };
    if blocked { return (StatusCode::TOO_MANY_REQUESTS, [("retry-after","1")], "Too many requests. Try again in one second.").into_response(); } next.run(req).await
}
#[derive(Debug)] struct AppError { status:StatusCode, message:String }
impl AppError { fn bad(message:&str)->Self{Self{status:StatusCode::BAD_REQUEST,message:message.into()}} fn not_found(message:&str)->Self{Self{status:StatusCode::NOT_FOUND,message:message.into()}} }
impl From<sqlx::Error> for AppError { fn from(_:sqlx::Error)->Self{Self{status:StatusCode::INTERNAL_SERVER_ERROR,message:"The packet store is unavailable. Try again shortly.".into()}} }
impl IntoResponse for AppError { fn into_response(self)->Response{(self.status,Json(serde_json::json!({"error":self.message}))).into_response()} }

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().json().with_env_filter("info").init();
    let port=env::var("PORT").unwrap_or_else(|_|"8080".into()); let build=env::var("BUILD_SHA").unwrap_or_else(|_|"dev".into());
    let db_url=env::var("DATABASE_URL").unwrap_or_else(|_|"sqlite:/data/diff-gate.db?mode=rwc".into());
    if !std::path::Path::new("/data").exists() { std::fs::create_dir_all("/data").ok(); }
    info!(config="generated DATABASE_URL default when absent", "Diff Gate starting");
    let db=SqlitePoolOptions::new().max_connections(5).connect(&db_url).await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS packets (id TEXT PRIMARY KEY, title TEXT NOT NULL, owner TEXT NOT NULL, status TEXT NOT NULL, data TEXT NOT NULL, created_at TEXT NOT NULL)").execute(&db).await?;
    let state=AppState{db,build};
    let api=Router::new().route("/health",get(health)).route("/api/packets",get(list_packets).post(create_packet)).route("/api/packets/:id",get(get_packet)).layer(middleware::from_fn(rate_limit));
    let index = || get_service(ServeFile::new("dist/index.html"));
    let app=Router::new().merge(api)
        .route("/", index()).route("/demo", index()).route("/privacy", index()).route("/terms", index()).route("/404", index())
        .nest_service("/assets", ServeDir::new("dist/assets"))
        .route_service("/favicon.svg", ServeFile::new("dist/favicon.svg")).route_service("/apple-touch-icon.png", ServeFile::new("dist/apple-touch-icon.png"))
        .route_service("/change-control.webp", ServeFile::new("dist/change-control.webp")).route_service("/social.webp", ServeFile::new("dist/social.webp"))
        .route_service("/robots.txt", ServeFile::new("dist/robots.txt")).route_service("/sitemap.xml", ServeFile::new("dist/sitemap.xml")).route_service("/manifest.webmanifest", ServeFile::new("dist/manifest.webmanifest"))
        .fallback_service(ServeFile::new("dist/index.html")).layer(TraceLayer::new_for_http())
        .layer(SetResponseHeaderLayer::overriding(HeaderName::from_static("x-content-type-options"), HeaderValue::from_static("nosniff")))
        .layer(SetResponseHeaderLayer::overriding(HeaderName::from_static("referrer-policy"), HeaderValue::from_static("strict-origin-when-cross-origin")))
        .layer(SetResponseHeaderLayer::overriding(HeaderName::from_static("content-security-policy"), HeaderValue::from_static("default-src 'self'; img-src 'self' data:; style-src 'self'; script-src 'self'; connect-src 'self' https://api.sociobot.in")))
        .with_state(state);
    let addr=SocketAddr::from(([0,0,0,0],port.parse()?)); info!(%addr,"listening"); let listener=tokio::net::TcpListener::bind(addr).await?; axum::serve(listener,app).with_graceful_shutdown(shutdown()).await?; Ok(())
}
async fn shutdown(){ let _=tokio::signal::ctrl_c().await; }
