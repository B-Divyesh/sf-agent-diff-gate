use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, HeaderName, HeaderValue, StatusCode},
    middleware,
    response::{IntoResponse, Redirect, Response},
    routing::{get, get_service, post},
    Json, Router,
};
use chrono::{Duration as ChronoDuration, Utc};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use reqwest::Client;
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
    trace::TraceLayer,
};
use tracing::info;
use url::Url;
use uuid::Uuid;

const RATE_LIMIT_PER_SECOND: u32 = 40;
const RETRY_AFTER_SECONDS: &str = "1";
const SESSION_DAYS: i64 = 14;

#[derive(Clone)]
struct AppState {
    db: SqlitePool,
    build: String,
    limits: Arc<Mutex<HashMap<String, Window>>>,
    github: GithubConfig,
    http: Client,
}
struct Window {
    since: Instant,
    hits: u32,
}
#[derive(Clone, Default)]
struct GithubConfig {
    client_id: Option<String>,
    client_secret: Option<String>,
    app_id: Option<String>,
    private_key: Option<String>,
    installation_id: Option<String>,
    app_slug: Option<String>,
    public_base: String,
}
impl GithubConfig {
    fn from_env() -> Self {
        Self {
            client_id: env::var("GITHUB_OAUTH_CLIENT_ID").ok(),
            client_secret: env::var("GITHUB_OAUTH_CLIENT_SECRET").ok(),
            app_id: env::var("GITHUB_APP_ID").ok(),
            private_key: env::var("GITHUB_APP_PRIVATE_KEY")
                .ok()
                .map(|v| v.replace("\\n", "\n")),
            installation_id: env::var("GITHUB_APP_INSTALLATION_ID").ok(),
            app_slug: env::var("GITHUB_APP_SLUG").ok(),
            public_base: env::var("PUBLIC_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:8080".into()),
        }
    }
    fn oauth_ready(&self) -> bool {
        self.client_id.is_some() && self.client_secret.is_some()
    }
    fn app_ready(&self) -> bool {
        self.app_id.is_some() && self.private_key.is_some() && self.installation_id.is_some()
    }
}
#[derive(Serialize)]
struct Health {
    status: &'static str,
    build: String,
}
#[derive(Serialize)]
struct AuthStatus {
    authenticated: bool,
    github_sign_in_configured: bool,
    github_app_configured: bool,
    install_url: Option<String>,
    user: Option<String>,
    team: Option<String>,
}
#[derive(Serialize, FromRow, Clone)]
struct Packet {
    id: String,
    title: String,
    owner: String,
    status: String,
    data: String,
    created_at: String,
    approved_by: Option<String>,
    approved_at: Option<String>,
    source_url: Option<String>,
}
#[derive(Deserialize)]
struct NewPacket {
    title: String,
    owner: Option<String>,
    data: serde_json::Value,
    source_url: Option<String>,
}
#[derive(Deserialize)]
struct Approval {
    note: Option<String>,
}
#[derive(Deserialize)]
struct ImportRequest {
    pr_url: String,
}
#[derive(Deserialize)]
struct GithubUser {
    login: String,
}
#[derive(Deserialize)]
struct GithubMembership {
    state: String,
    organization: GithubOrganization,
}
#[derive(Deserialize)]
struct GithubOrganization {
    login: String,
}
#[derive(Deserialize)]
struct GithubPull {
    title: String,
    html_url: String,
    user: GithubPullUser,
}
#[derive(Deserialize)]
struct GithubPullUser {
    login: String,
}
#[derive(Deserialize)]
struct GithubFile {
    filename: String,
}
#[derive(Serialize)]
struct GithubJwtClaims {
    iss: String,
    iat: usize,
    exp: usize,
}
#[derive(FromRow)]
struct Session {
    team_id: String,
    login: String,
    team_name: String,
}

async fn health(State(state): State<AppState>) -> Json<Health> {
    Json(Health {
        status: "ok",
        build: state.build,
    })
}
fn cookie_value(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .find_map(|part| {
            let (key, value) = part.trim().split_once('=')?;
            (key == name).then(|| value.to_string())
        })
}
async fn session(state: &AppState, headers: &HeaderMap) -> Result<Session, AppError> {
    let token = cookie_value(headers, "diff_gate_session").ok_or_else(|| {
        AppError::unauthorized("Sign in with GitHub before opening team packets.")
    })?;
    sqlx::query_as::<_, Session>("SELECT s.team_id, s.login, t.name AS team_name FROM sessions s JOIN teams t ON t.id=s.team_id WHERE s.token=? AND s.expires_at > ?").bind(token).bind(Utc::now().to_rfc3339()).fetch_optional(&state.db).await?.ok_or_else(|| AppError::unauthorized("Your session ended. Sign in with GitHub again."))
}
async fn auth_status(State(state): State<AppState>, headers: HeaderMap) -> Json<AuthStatus> {
    let active = session(&state, &headers).await.ok();
    Json(AuthStatus {
        authenticated: active.is_some(),
        github_sign_in_configured: state.github.oauth_ready(),
        github_app_configured: state.github.app_ready(),
        install_url: state
            .github
            .app_slug
            .as_ref()
            .map(|slug| format!("https://github.com/apps/{slug}/installations/new")),
        user: active.as_ref().map(|s| s.login.clone()),
        team: active.map(|s| s.team_name),
    })
}
async fn github_login(State(state): State<AppState>) -> Result<Redirect, AppError> {
    if !state.github.oauth_ready() {
        return Err(AppError::service_unavailable(
            "GitHub sign-in is not configured on this deployment.",
        ));
    }
    let nonce = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO oauth_states (state,created_at) VALUES (?,?)")
        .bind(&nonce)
        .bind(Utc::now().to_rfc3339())
        .execute(&state.db)
        .await?;
    let callback = format!(
        "{}/auth/github/callback",
        state.github.public_base.trim_end_matches('/')
    );
    let mut url = Url::parse("https://github.com/login/oauth/authorize").expect("GitHub OAuth URL");
    url.query_pairs_mut()
        .append_pair(
            "client_id",
            state.github.client_id.as_deref().unwrap_or_default(),
        )
        .append_pair("redirect_uri", &callback)
        .append_pair("scope", "read:user read:org")
        .append_pair("state", &nonce);
    Ok(Redirect::temporary(url.as_str()))
}
#[derive(Deserialize)]
struct OAuthCallback {
    code: String,
    state: String,
}
async fn github_callback(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<OAuthCallback>,
) -> Result<Response, AppError> {
    if !state.github.oauth_ready() {
        return Err(AppError::service_unavailable(
            "GitHub sign-in is not configured on this deployment.",
        ));
    }
    if sqlx::query("DELETE FROM oauth_states WHERE state=?")
        .bind(&query.state)
        .execute(&state.db)
        .await?
        .rows_affected()
        != 1
    {
        return Err(AppError::bad(
            "That GitHub sign-in link expired. Start again.",
        ));
    }
    let token = state.http.post("https://github.com/login/oauth/access_token").header("accept", "application/json").json(&serde_json::json!({"client_id":state.github.client_id,"client_secret":state.github.client_secret,"code":query.code})).send().await.map_err(|_| AppError::service_unavailable("GitHub could not complete sign-in. Try again."))?.error_for_status().map_err(|_| AppError::service_unavailable("GitHub could not complete sign-in. Try again."))?.json::<serde_json::Value>().await.map_err(|_| AppError::service_unavailable("GitHub returned an invalid sign-in response."))?;
    let access_token = token
        .get("access_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::service_unavailable("GitHub did not return an access token."))?;
    let user = state
        .http
        .get("https://api.github.com/user")
        .header("user-agent", "diff-gate")
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|_| AppError::service_unavailable("Could not read your GitHub identity."))?
        .error_for_status()
        .map_err(|_| AppError::service_unavailable("Could not read your GitHub identity."))?
        .json::<GithubUser>()
        .await
        .map_err(|_| AppError::service_unavailable("GitHub returned an invalid user profile."))?;
    let memberships = match state
        .http
        .get("https://api.github.com/user/memberships/orgs?state=active")
        .header("user-agent", "diff-gate")
        .bearer_auth(access_token)
        .send()
        .await
    {
        Ok(response) => match response.error_for_status() {
            Ok(response) => response.json::<Vec<GithubMembership>>().await.ok(),
            Err(_) => None,
        },
        Err(_) => None,
    };
    let organization = memberships
        .as_deref()
        .and_then(|memberships| {
            memberships
                .iter()
                .find(|membership| membership.state == "active")
        })
        .map(|membership| membership.organization.login.clone());
    let team_key = organization.clone().unwrap_or_else(|| user.login.clone());
    let team_id = format!("github:{}", team_key.to_lowercase());
    let team_name = organization.unwrap_or_else(|| format!("{}'s private workspace", user.login));
    sqlx::query("INSERT OR IGNORE INTO teams (id,name,created_at) VALUES (?,?,?)")
        .bind(&team_id)
        .bind(team_name)
        .bind(Utc::now().to_rfc3339())
        .execute(&state.db)
        .await?;
    let session_token = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO sessions (token,team_id,login,expires_at) VALUES (?,?,?,?)")
        .bind(&session_token)
        .bind(&team_id)
        .bind(&user.login)
        .bind((Utc::now() + ChronoDuration::days(SESSION_DAYS)).to_rfc3339())
        .execute(&state.db)
        .await?;
    Ok(([(header::SET_COOKIE, format!("diff_gate_session={session_token}; Path=/; HttpOnly; SameSite=Lax; Secure; Max-Age={}", SESSION_DAYS*86400))], Redirect::to("/")).into_response())
}
async fn sign_out(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Some(token) = cookie_value(&headers, "diff_gate_session") {
        let _ = sqlx::query("DELETE FROM sessions WHERE token=?")
            .bind(token)
            .execute(&state.db)
            .await;
    }
    (
        [(
            header::SET_COOKIE,
            "diff_gate_session=; Path=/; HttpOnly; SameSite=Lax; Secure; Max-Age=0",
        )],
        StatusCode::NO_CONTENT,
    )
        .into_response()
}

async fn list_packets(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<Packet>>, AppError> {
    let who = session(&state, &headers).await?;
    Ok(Json(sqlx::query_as("SELECT id,title,owner,status,data,created_at,approved_by,approved_at,source_url FROM packets WHERE team_id=? ORDER BY created_at DESC LIMIT 50").bind(who.team_id).fetch_all(&state.db).await?))
}
async fn create_packet(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<NewPacket>,
) -> Result<(StatusCode, Json<Packet>), AppError> {
    let who = session(&state, &headers).await?;
    if input.title.trim().is_empty() || input.title.len() > 180 {
        return Err(AppError::bad("Add a title before saving."));
    }
    let owner = input
        .owner
        .as_deref()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or(&who.login)
        .trim()
        .to_string();
    let packet = Packet {
        id: Uuid::new_v4().to_string(),
        title: input.title.trim().to_string(),
        owner,
        status: "needs review".into(),
        data: input.data.to_string(),
        created_at: Utc::now().to_rfc3339(),
        approved_by: None,
        approved_at: None,
        source_url: input.source_url,
    };
    sqlx::query("INSERT INTO packets (id,team_id,title,owner,status,data,created_at,approved_by,approved_at,source_url) VALUES (?,?,?,?,?,?,?,?,?,?)").bind(&packet.id).bind(&who.team_id).bind(&packet.title).bind(&packet.owner).bind(&packet.status).bind(&packet.data).bind(&packet.created_at).bind(&packet.approved_by).bind(&packet.approved_at).bind(&packet.source_url).execute(&state.db).await?;
    audit(
        &state.db,
        &packet.id,
        &who.login,
        "created",
        "Review packet created.",
    )
    .await?;
    Ok((StatusCode::CREATED, Json(packet)))
}
async fn get_packet(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Packet>, AppError> {
    let who = session(&state, &headers).await?;
    sqlx::query_as("SELECT id,title,owner,status,data,created_at,approved_by,approved_at,source_url FROM packets WHERE id=? AND team_id=?").bind(id).bind(who.team_id).fetch_optional(&state.db).await?.map(Json).ok_or_else(|| AppError::not_found("That review packet was not found in this team."))
}
async fn approve_packet(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<Approval>,
) -> Result<Json<Packet>, AppError> {
    let who = session(&state, &headers).await?;
    let updated = sqlx::query(
        "UPDATE packets SET status='approved',approved_by=?,approved_at=? WHERE id=? AND team_id=?",
    )
    .bind(&who.login)
    .bind(Utc::now().to_rfc3339())
    .bind(&id)
    .bind(&who.team_id)
    .execute(&state.db)
    .await?
    .rows_affected();
    if updated != 1 {
        return Err(AppError::not_found(
            "That review packet was not found in this team.",
        ));
    }
    audit(
        &state.db,
        &id,
        &who.login,
        "approved",
        input
            .note
            .as_deref()
            .unwrap_or("Owner approved this packet."),
    )
    .await?;
    get_packet(State(state), headers, Path(id)).await
}
async fn audit(
    db: &SqlitePool,
    packet_id: &str,
    actor: &str,
    action: &str,
    detail: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO packet_audit (id,packet_id,actor,action,detail,created_at) VALUES (?,?,?,?,?,?)").bind(Uuid::new_v4().to_string()).bind(packet_id).bind(actor).bind(action).bind(detail).bind(Utc::now().to_rfc3339()).execute(db).await.map(|_| ())
}

fn parse_pr_url(raw: &str) -> Result<(String, String, u64), AppError> {
    let url = Url::parse(raw).map_err(|_| AppError::bad("Paste a GitHub pull request URL."))?;
    if url.host_str() != Some("github.com") {
        return Err(AppError::bad("Use a github.com pull request URL."));
    }
    let parts: Vec<_> = url.path_segments().map(|v| v.collect()).unwrap_or_default();
    if parts.len() != 4 || parts[2] != "pull" {
        return Err(AppError::bad(
            "Use a GitHub URL shaped owner/repo/pull/123.",
        ));
    }
    let number = parts[3]
        .parse()
        .map_err(|_| AppError::bad("The pull request number must be a number."))?;
    Ok((parts[0].to_string(), parts[1].to_string(), number))
}
async fn installation_token(state: &AppState) -> Result<String, AppError> {
    if !state.github.app_ready() {
        return Err(AppError::service_unavailable(
            "Connect the Diff Gate GitHub App before importing a pull request.",
        ));
    }
    let now = Utc::now().timestamp() as usize;
    let jwt = encode(
        &Header::new(Algorithm::RS256),
        &GithubJwtClaims {
            iss: state.github.app_id.clone().unwrap_or_default(),
            iat: now.saturating_sub(30),
            exp: now + 540,
        },
        &EncodingKey::from_rsa_pem(
            state
                .github
                .private_key
                .as_deref()
                .unwrap_or_default()
                .as_bytes(),
        )
        .map_err(|_| AppError::service_unavailable("The GitHub App key is invalid."))?,
    )
    .map_err(|_| AppError::service_unavailable("Could not sign the GitHub App request."))?;
    let endpoint = format!(
        "https://api.github.com/app/installations/{}/access_tokens",
        state.github.installation_id.as_deref().unwrap_or_default()
    );
    let value = state
        .http
        .post(endpoint)
        .header("user-agent", "diff-gate")
        .header("accept", "application/vnd.github+json")
        .bearer_auth(jwt)
        .send()
        .await
        .map_err(|_| {
            AppError::service_unavailable("GitHub App could not request repository access.")
        })?
        .error_for_status()
        .map_err(|_| {
            AppError::service_unavailable("GitHub App access was denied. Check the installation.")
        })?
        .json::<serde_json::Value>()
        .await
        .map_err(|_| {
            AppError::service_unavailable("GitHub returned an invalid installation token.")
        })?;
    value
        .get("token")
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .ok_or_else(|| {
            AppError::service_unavailable("GitHub App did not return an installation token.")
        })
}
async fn import_github_pr(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<ImportRequest>,
) -> Result<(StatusCode, Json<Packet>), AppError> {
    let who = session(&state, &headers).await?;
    let (owner, repo, number) = parse_pr_url(&input.pr_url)?;
    let token = installation_token(&state).await?;
    let base = format!("https://api.github.com/repos/{owner}/{repo}/pulls/{number}");
    let pull = state
        .http
        .get(&base)
        .header("user-agent", "diff-gate")
        .header("accept", "application/vnd.github+json")
        .bearer_auth(&token)
        .send()
        .await
        .map_err(|_| AppError::service_unavailable("GitHub could not load that pull request."))?
        .error_for_status()
        .map_err(|_| {
            AppError::bad("GitHub could not find that pull request for this App installation.")
        })?
        .json::<GithubPull>()
        .await
        .map_err(|_| AppError::service_unavailable("GitHub returned an invalid pull request."))?;
    let files = state
        .http
        .get(format!("{base}/files?per_page=100"))
        .header("user-agent", "diff-gate")
        .header("accept", "application/vnd.github+json")
        .bearer_auth(&token)
        .send()
        .await
        .map_err(|_| AppError::service_unavailable("GitHub could not load changed files."))?
        .error_for_status()
        .map_err(|_| AppError::service_unavailable("GitHub could not load changed files."))?
        .json::<Vec<GithubFile>>()
        .await
        .map_err(|_| AppError::service_unavailable("GitHub returned invalid changed files."))?;
    let changed: Vec<String> = files.iter().map(|f| f.filename.clone()).collect();
    let has_migration = changed
        .iter()
        .any(|f| f.contains("migration") || f.starts_with("db/"));
    let has_contract = changed
        .iter()
        .any(|f| f.contains("api/") || f.contains("contract") || f.ends_with(".graphql"));
    let checks = serde_json::json!([{"label":"Pull request imported","detail":format!("PR #{number} by {}. {} changed files.", pull.user.login, changed.len()),"state":"ready"},{"label":"Contract changed","detail":if has_contract {"API or contract path changed. Confirm downstream compatibility."} else {"No contract path matched the default policy."},"state":if has_contract {"risk"} else {"ready"}},{"label":"Migration found","detail":if has_migration {"Migration path changed. Database owner sign-off is required."} else {"No migration path matched the default policy."},"state":if has_migration {"risk"} else {"ready"}},{"label":"Test evidence","detail":"Attach the test command and result before owner approval.","state":"missing"}]);
    let data = serde_json::json!({"source":format!("PR #{number} · GitHub App import"),"changed":changed,"checks":checks,"policy":"Default policy: contracts and migrations require named owner review."});
    let packet = Packet {
        id: Uuid::new_v4().to_string(),
        title: pull.title,
        owner: who.login.clone(),
        status: "needs review".into(),
        data: data.to_string(),
        created_at: Utc::now().to_rfc3339(),
        approved_by: None,
        approved_at: None,
        source_url: Some(pull.html_url),
    };
    sqlx::query("INSERT INTO packets (id,team_id,title,owner,status,data,created_at,approved_by,approved_at,source_url) VALUES (?,?,?,?,?,?,?,?,?,?)").bind(&packet.id).bind(&who.team_id).bind(&packet.title).bind(&packet.owner).bind(&packet.status).bind(&packet.data).bind(&packet.created_at).bind(&packet.approved_by).bind(&packet.approved_at).bind(&packet.source_url).execute(&state.db).await?;
    audit(
        &state.db,
        &packet.id,
        &who.login,
        "imported",
        "GitHub App imported this pull request and evaluated the default policy.",
    )
    .await?;
    Ok((StatusCode::CREATED, Json(packet)))
}

fn client_ip(headers: &HeaderMap) -> String {
    headers
        .get("x-forwarded-for")
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(str::trim)
        .filter(|v| !v.is_empty())
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
        let now = Instant::now();
        limits.retain(|_, window| now.duration_since(window.since) < Duration::from_secs(60));
        let window = limits.entry(ip).or_insert(Window {
            since: now,
            hits: 0,
        });
        if now.duration_since(window.since) >= Duration::from_secs(1) {
            window.since = now;
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
async fn cache_headers(request: axum::extract::Request, next: middleware::Next) -> Response {
    let path = request.uri().path().to_string();
    let mut response = next.run(request).await;
    let value = if path.starts_with("/assets/") || path.ends_with(".webp") || path.ends_with(".png")
    {
        "public, max-age=31536000, immutable"
    } else {
        "no-cache"
    };
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static(value));
    response
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
    fn unauthorized(message: &str) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            message: message.into(),
        }
    }
    fn not_found(message: &str) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: message.into(),
        }
    }
    fn service_unavailable(message: &str) -> Self {
        Self {
            status: StatusCode::SERVICE_UNAVAILABLE,
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
        (self.status, Json(serde_json::json!({"error":self.message}))).into_response()
    }
}
async fn not_found_page() -> Response {
    match tokio::fs::read("dist/index.html").await {
        Ok(body) => (
            StatusCode::NOT_FOUND,
            [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
            body,
        )
            .into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "Not found").into_response(),
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
        .fallback(get(not_found_page))
}
fn app(state: AppState) -> Router {
    let protected = Router::new()
        .route("/api/packets", get(list_packets).post(create_packet))
        .route("/api/packets/:id", get(get_packet))
        .route("/api/packets/:id/approve", post(approve_packet))
        .route("/api/github/import", post(import_github_pr))
        .route("/api/auth/status", get(auth_status))
        .route("/auth/github", get(github_login))
        .route("/auth/github/callback", get(github_callback))
        .route("/api/auth/signout", post(sign_out))
        .merge(static_routes())
        .layer(middleware::from_fn(cache_headers))
        .layer(middleware::from_fn_with_state(state.clone(), rate_limit));
    Router::new()
        .route("/health", get(health))
        .merge(protected)
        .layer(TraceLayer::new_for_http())
        .layer(middleware::from_fn(security_headers))
        .with_state(state)
}
async fn security_headers(request: axum::extract::Request, next: middleware::Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(
        HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    headers.insert(HeaderName::from_static("content-security-policy"),HeaderValue::from_static("default-src 'self'; img-src 'self' data:; style-src 'self'; script-src 'self'; connect-src 'self' https://api.sociobot.in https://github.com https://api.github.com; frame-ancestors 'none'"));
    response
}
async fn create_schema(db: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query("CREATE TABLE IF NOT EXISTS teams (id TEXT PRIMARY KEY,name TEXT NOT NULL,created_at TEXT NOT NULL)").execute(db).await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS sessions (token TEXT PRIMARY KEY,team_id TEXT NOT NULL,login TEXT NOT NULL,expires_at TEXT NOT NULL)").execute(db).await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS oauth_states (state TEXT PRIMARY KEY,created_at TEXT NOT NULL)",
    )
    .execute(db)
    .await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS packets (id TEXT PRIMARY KEY,team_id TEXT NOT NULL DEFAULT '',title TEXT NOT NULL,owner TEXT NOT NULL,status TEXT NOT NULL,data TEXT NOT NULL,created_at TEXT NOT NULL,approved_by TEXT,approved_at TEXT,source_url TEXT)").execute(db).await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS packet_audit (id TEXT PRIMARY KEY,packet_id TEXT NOT NULL,actor TEXT NOT NULL,action TEXT NOT NULL,detail TEXT NOT NULL,created_at TEXT NOT NULL)").execute(db).await?;
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
    let github = GithubConfig::from_env();
    info!(
        database_config,
        github_oauth = github.oauth_ready(),
        github_app = github.app_ready(),
        "Diff Gate starting; only optional GitHub configuration was supplied"
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
        github,
        http: Client::new(),
    };
    let address = SocketAddr::from(([0, 0, 0, 0], port.parse()?));
    info!(%address,"listening");
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
    async fn test_app() -> (Router, SqlitePool) {
        let db = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        create_schema(&db).await.unwrap();
        let state = AppState {
            db: db.clone(),
            build: "test-sha".into(),
            limits: Arc::new(Mutex::new(HashMap::new())),
            github: GithubConfig::default(),
            http: Client::new(),
        };
        (app(state), db)
    }
    #[tokio::test]
    async fn health_reports_the_build_identity() {
        let (app, _) = test_app().await;
        let response = app
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
    async fn packets_require_an_authenticated_team_session() {
        let (app, _) = test_app().await;
        let response = app
            .oneshot(
                Request::get("/api/packets")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
    #[tokio::test]
    async fn packet_reads_and_approvals_are_scoped_to_the_signed_in_team() {
        let (app, db) = test_app().await;
        let now = (Utc::now() + ChronoDuration::days(1)).to_rfc3339();
        sqlx::query("INSERT INTO teams (id,name,created_at) VALUES ('a','A',?),('b','B',?)")
            .bind(&now)
            .bind(&now)
            .execute(&db)
            .await
            .unwrap();
        sqlx::query("INSERT INTO sessions (token,team_id,login,expires_at) VALUES ('a-token','a','alice',?),('b-token','b','bea',?)")
            .bind(&now)
            .bind(&now)
            .execute(&db)
            .await
            .unwrap();
        sqlx::query("INSERT INTO packets (id,team_id,title,owner,status,data,created_at) VALUES ('packet-b','b','Private change','bea','needs review','{}',?)")
            .bind(&now)
            .execute(&db)
            .await
            .unwrap();
        let hidden = app
            .clone()
            .oneshot(
                Request::get("/api/packets/packet-b")
                    .header("cookie", "diff_gate_session=a-token")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(hidden.status(), StatusCode::NOT_FOUND);
        let forbidden_approval = app
            .clone()
            .oneshot(
                Request::post("/api/packets/packet-b/approve")
                    .header("cookie", "diff_gate_session=a-token")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(forbidden_approval.status(), StatusCode::NOT_FOUND);
        let approved = app
            .oneshot(
                Request::post("/api/packets/packet-b/approve")
                    .header("cookie", "diff_gate_session=b-token")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from("{\"note\":\"Reviewed evidence\"}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(approved.status(), StatusCode::OK);
        let audit_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM packet_audit WHERE packet_id='packet-b' AND actor='bea' AND action='approved'").fetch_one(&db).await.unwrap();
        assert_eq!(audit_count, 1);
    }
    #[tokio::test]
    async fn rate_limit_uses_the_first_forwarded_ip_and_returns_retry_after() {
        let (app, _) = test_app().await;
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
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
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
    fn github_urls_are_strict_and_docker_uses_the_unpinned_rust_major() {
        assert_eq!(
            parse_pr_url("https://github.com/acme/api/pull/42").unwrap(),
            ("acme".into(), "api".into(), 42)
        );
        assert!(parse_pr_url("https://example.com/acme/api/pull/42").is_err());
        let dockerfile = include_str!("../../Dockerfile");
        assert!(dockerfile.contains("FROM rust:1-alpine AS build"));
        assert!(dockerfile.contains("EXPOSE 8080"));
    }
}
