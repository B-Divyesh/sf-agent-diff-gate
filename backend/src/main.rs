use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, HeaderName, HeaderValue, StatusCode},
    middleware,
    response::{IntoResponse, Redirect, Response},
    routing::{get, get_service, post},
    Json, Router,
};
use chrono::{Duration as ChronoDuration, Utc};
use jsonwebtoken::{
    decode, decode_header, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation,
};
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
const DEFAULT_RETENTION_DAYS: i64 = 90;
const MAX_RETENTION_DAYS: i64 = 3650;

#[derive(Clone)]
struct AppState {
    db: SqlitePool,
    build: String,
    limits: Arc<Mutex<HashMap<String, Window>>>,
    identity: EntraConfig,
    github: GithubConfig,
    http: Client,
}
struct Window {
    since: Instant,
    hits: u32,
}
#[derive(Clone, Default)]
struct GithubConfig {
    app_id: Option<String>,
    private_key: Option<String>,
    app_slug: Option<String>,
    installations: HashMap<String, String>,
}
#[derive(Clone, Default)]
struct EntraConfig {
    authority: Option<String>,
    client_id: Option<String>,
    client_secret: Option<String>,
    public_base: String,
    team_claim: String,
}
impl GithubConfig {
    fn from_env() -> Self {
        Self {
            app_id: env::var("GITHUB_APP_ID").ok(),
            private_key: env::var("GITHUB_APP_PRIVATE_KEY")
                .ok()
                .map(|v| v.replace("\\n", "\n")),
            app_slug: env::var("GITHUB_APP_SLUG").ok(),
            installations: env::var("GITHUB_TEAM_INSTALLATIONS")
                .ok()
                .and_then(|raw| serde_json::from_str(&raw).ok())
                .unwrap_or_default(),
        }
    }
    fn app_ready(&self) -> bool {
        self.app_id.is_some() && self.private_key.is_some() && !self.installations.is_empty()
    }
    fn installation_for(&self, team_id: &str) -> Option<String> {
        self.installations.get(team_id).cloned()
    }
}
impl EntraConfig {
    fn from_env() -> Self {
        Self {
            authority: env::var("ENTRA_AUTHORITY")
                .ok()
                .and_then(|value| sociobot_entra_authority(&value)),
            client_id: env::var("ENTRA_CLIENT_ID").ok(),
            client_secret: env::var("ENTRA_CLIENT_SECRET").ok(),
            public_base: env::var("PUBLIC_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:8080".into()),
            team_claim: env::var("ENTRA_TEAM_CLAIM")
                .unwrap_or_else(|_| "extension_DiffGateTeam".into()),
        }
    }
    fn ready(&self) -> bool {
        self.authority
            .as_deref()
            .and_then(sociobot_entra_authority)
            .is_some()
            && self.client_id.is_some()
            && self.client_secret.is_some()
    }
    fn callback_url(&self) -> String {
        format!(
            "{}/auth/entra/callback",
            self.public_base.trim_end_matches('/')
        )
    }
}
fn sociobot_entra_authority(raw: &str) -> Option<String> {
    let value = raw.trim_end_matches('/');
    let url = Url::parse(value).ok()?;
    (url.scheme() == "https"
        && url.host_str() == Some("sociobotcustomers.ciamlogin.com")
        && url.port_or_known_default() == Some(443)
        && url.username().is_empty()
        && url.password().is_none()
        && url
            .path_segments()
            .is_some_and(|mut parts| parts.next().is_some_and(|tenant| !tenant.is_empty()))
        && url.query().is_none()
        && url.fragment().is_none())
    .then(|| value.to_string())
}
#[derive(Serialize)]
struct Health {
    status: &'static str,
    build: String,
}
#[derive(Serialize)]
struct AuthStatus {
    authenticated: bool,
    entra_sign_in_configured: bool,
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
#[derive(Serialize, FromRow)]
struct AuditEntry {
    id: String,
    actor: String,
    action: String,
    detail: String,
    created_at: String,
}
#[derive(Serialize)]
struct TeamSettings {
    retention_days: i64,
}
#[derive(Deserialize)]
struct SettingsUpdate {
    retention_days: i64,
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
struct EvidenceUpdate {
    data: serde_json::Value,
}
#[derive(Deserialize)]
struct ImportRequest {
    pr_url: String,
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
#[derive(Deserialize)]
struct EntraMetadata {
    issuer: String,
    jwks_uri: String,
}
#[derive(Deserialize)]
struct EntraClaims {
    sub: String,
    oid: Option<String>,
    preferred_username: Option<String>,
    name: Option<String>,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
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

async fn validate_entra_token(state: &AppState, id_token: &str) -> Result<EntraClaims, AppError> {
    let authority = state.identity.authority.as_deref().unwrap_or_default();
    let metadata = state
        .http
        .get(format!("{authority}/v2.0/.well-known/openid-configuration"))
        .send()
        .await
        .map_err(|_| {
            AppError::service_unavailable("Could not validate the Sociobot identity token.")
        })?
        .error_for_status()
        .map_err(|_| {
            AppError::service_unavailable("Could not validate the Sociobot identity token.")
        })?
        .json::<EntraMetadata>()
        .await
        .map_err(|_| AppError::service_unavailable("Sociobot identity metadata was invalid."))?;
    let header = decode_header(id_token)
        .map_err(|_| AppError::unauthorized("Sociobot returned an invalid identity token."))?;
    let kid = header
        .kid
        .ok_or_else(|| AppError::unauthorized("Sociobot identity token has no signing key."))?;
    let keys = state
        .http
        .get(&metadata.jwks_uri)
        .send()
        .await
        .map_err(|_| {
            AppError::service_unavailable("Could not validate the Sociobot identity token.")
        })?
        .error_for_status()
        .map_err(|_| {
            AppError::service_unavailable("Could not validate the Sociobot identity token.")
        })?
        .json::<jsonwebtoken::jwk::JwkSet>()
        .await
        .map_err(|_| AppError::service_unavailable("Sociobot signing keys were invalid."))?;
    let jwk = keys
        .keys
        .into_iter()
        .find(|key| key.common.key_id.as_deref() == Some(&kid))
        .ok_or_else(|| AppError::unauthorized("Sociobot signing key was not recognized."))?;
    let key = DecodingKey::from_jwk(&jwk)
        .map_err(|_| AppError::unauthorized("Sociobot signing key was invalid."))?;
    let mut validation = Validation::new(header.alg);
    validation.set_audience(&[state.identity.client_id.as_deref().unwrap_or_default()]);
    validation.set_issuer(&[metadata.issuer]);
    decode::<EntraClaims>(id_token, &key, &validation)
        .map(|data| data.claims)
        .map_err(|_| AppError::unauthorized("Sociobot identity token validation failed."))
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
        AppError::unauthorized("Sign in with Sociobot before opening team packets.")
    })?;
    sqlx::query_as::<_, Session>("SELECT s.team_id, s.login, t.name AS team_name FROM sessions s JOIN teams t ON t.id=s.team_id WHERE s.token=? AND s.expires_at > ?").bind(token).bind(Utc::now().to_rfc3339()).fetch_optional(&state.db).await?.ok_or_else(|| AppError::unauthorized("Your session ended. Sign in with Sociobot again."))
}
async fn auth_status(State(state): State<AppState>, headers: HeaderMap) -> Json<AuthStatus> {
    let _ = purge_expired_transient_rows(&state.db).await;
    let active = session(&state, &headers).await.ok();
    Json(AuthStatus {
        authenticated: active.is_some(),
        entra_sign_in_configured: state.identity.ready(),
        github_app_configured: active
            .as_ref()
            .and_then(|session| state.github.installation_for(&session.team_id))
            .is_some()
            && state.github.app_ready(),
        install_url: state
            .github
            .app_slug
            .as_ref()
            .map(|slug| format!("https://github.com/apps/{slug}/installations/new")),
        user: active.as_ref().map(|s| s.login.clone()),
        team: active.map(|s| s.team_name),
    })
}
async fn entra_login(State(state): State<AppState>) -> Result<Redirect, AppError> {
    if !state.identity.ready() {
        return Err(AppError::service_unavailable(
            "Sociobot Entra sign-in is not configured on this deployment.",
        ));
    }
    purge_expired_transient_rows(&state.db).await?;
    let nonce = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO oauth_states (state,created_at) VALUES (?,?)")
        .bind(&nonce)
        .bind(Utc::now().to_rfc3339())
        .execute(&state.db)
        .await?;
    let authority = state.identity.authority.as_deref().unwrap_or_default();
    let mut url = Url::parse(&format!("{authority}/oauth2/v2.0/authorize"))
        .map_err(|_| AppError::service_unavailable("The Entra authority is invalid."))?;
    url.query_pairs_mut()
        .append_pair(
            "client_id",
            state.identity.client_id.as_deref().unwrap_or_default(),
        )
        .append_pair("redirect_uri", &state.identity.callback_url())
        .append_pair("response_type", "code")
        .append_pair("response_mode", "query")
        .append_pair("scope", "openid profile email")
        .append_pair("state", &nonce)
        .append_pair("nonce", &nonce);
    Ok(Redirect::temporary(url.as_str()))
}
#[derive(Deserialize)]
struct OAuthCallback {
    code: String,
    state: String,
}
async fn entra_callback(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<OAuthCallback>,
) -> Result<Response, AppError> {
    if !state.identity.ready() {
        return Err(AppError::service_unavailable(
            "Sociobot Entra sign-in is not configured on this deployment.",
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
            "That Sociobot sign-in link expired. Start again.",
        ));
    }
    let authority = state.identity.authority.as_deref().unwrap_or_default();
    let token = state
        .http
        .post(format!("{authority}/oauth2/v2.0/token"))
        .form(&[
            (
                "client_id",
                state.identity.client_id.as_deref().unwrap_or_default(),
            ),
            (
                "client_secret",
                state.identity.client_secret.as_deref().unwrap_or_default(),
            ),
            ("code", &query.code),
            ("redirect_uri", &state.identity.callback_url()),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await
        .map_err(|_| {
            AppError::service_unavailable("Sociobot could not complete sign-in. Try again.")
        })?
        .error_for_status()
        .map_err(|_| {
            AppError::service_unavailable("Sociobot could not complete sign-in. Try again.")
        })?
        .json::<serde_json::Value>()
        .await
        .map_err(|_| {
            AppError::service_unavailable("Sociobot returned an invalid sign-in response.")
        })?;
    let id_token = token
        .get("id_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            AppError::service_unavailable("Sociobot did not return an identity token.")
        })?;
    let claims = validate_entra_token(&state, id_token).await?;
    let team_value = claims.extra.get(&state.identity.team_claim).and_then(|value| match value { serde_json::Value::String(value) => Some(value.clone()), serde_json::Value::Array(values) => values.first().and_then(|v| v.as_str()).map(str::to_string), _ => None }).filter(|value| !value.trim().is_empty()).ok_or_else(|| AppError::forbidden("Your Sociobot account has no Diff Gate team claim. Ask a team administrator to assign one."))?;
    let team_id = format!("entra:{team_value}");
    let login = claims
        .preferred_username
        .or(claims.name)
        .or(claims.oid)
        .unwrap_or(claims.sub);
    let team_name = format!("Sociobot team {team_value}");
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
        .bind(&login)
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
    purge_team_packets(&state.db, &who.team_id).await?;
    Ok(Json(sqlx::query_as("SELECT id,title,owner,status,data,created_at,approved_by,approved_at,source_url FROM packets WHERE team_id=? ORDER BY created_at DESC LIMIT 50").bind(who.team_id).fetch_all(&state.db).await?))
}
async fn list_packet_audit(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Vec<AuditEntry>>, AppError> {
    let who = session(&state, &headers).await?;
    purge_team_packets(&state.db, &who.team_id).await?;
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM packets WHERE id=? AND team_id=?)")
            .bind(&id)
            .bind(&who.team_id)
            .fetch_one(&state.db)
            .await?;
    if !exists {
        return Err(AppError::not_found(
            "That review packet was not found in this team.",
        ));
    }
    Ok(Json(sqlx::query_as("SELECT id,actor,action,detail,created_at FROM packet_audit WHERE packet_id=? ORDER BY created_at,id").bind(id).fetch_all(&state.db).await?))
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
    purge_team_packets(&state.db, &who.team_id).await?;
    sqlx::query_as("SELECT id,title,owner,status,data,created_at,approved_by,approved_at,source_url FROM packets WHERE id=? AND team_id=?").bind(id).bind(who.team_id).fetch_optional(&state.db).await?.map(Json).ok_or_else(|| AppError::not_found("That review packet was not found in this team."))
}
async fn delete_packet(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let who = session(&state, &headers).await?;
    let mut transaction = state.db.begin().await?;
    let owned: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM packets WHERE id=? AND team_id=?)")
            .bind(&id)
            .bind(&who.team_id)
            .fetch_one(&mut *transaction)
            .await?;
    if !owned {
        return Err(AppError::not_found(
            "That review packet was not found in this team.",
        ));
    }
    sqlx::query("DELETE FROM packet_audit WHERE packet_id=?")
        .bind(&id)
        .execute(&mut *transaction)
        .await?;
    sqlx::query("DELETE FROM packets WHERE id=? AND team_id=?")
        .bind(&id)
        .bind(&who.team_id)
        .execute(&mut *transaction)
        .await?;
    transaction.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<TeamSettings>, AppError> {
    let who = session(&state, &headers).await?;
    Ok(Json(TeamSettings {
        retention_days: retention_days(&state.db, &who.team_id).await?,
    }))
}

async fn update_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<SettingsUpdate>,
) -> Result<Json<TeamSettings>, AppError> {
    let who = session(&state, &headers).await?;
    if !(1..=MAX_RETENTION_DAYS).contains(&input.retention_days) {
        return Err(AppError::bad("Choose retention between 1 and 3,650 days."));
    }
    sqlx::query("INSERT INTO team_settings (team_id,retention_days) VALUES (?,?) ON CONFLICT(team_id) DO UPDATE SET retention_days=excluded.retention_days")
        .bind(&who.team_id)
        .bind(input.retention_days)
        .execute(&state.db)
        .await?;
    purge_team_packets(&state.db, &who.team_id).await?;
    Ok(Json(TeamSettings {
        retention_days: input.retention_days,
    }))
}

async fn retention_days(db: &SqlitePool, team_id: &str) -> Result<i64, sqlx::Error> {
    Ok(
        sqlx::query_scalar::<_, i64>("SELECT retention_days FROM team_settings WHERE team_id=?")
            .bind(team_id)
            .fetch_optional(db)
            .await?
            .unwrap_or(DEFAULT_RETENTION_DAYS),
    )
}

async fn purge_team_packets(db: &SqlitePool, team_id: &str) -> Result<(), sqlx::Error> {
    let days = retention_days(db, team_id).await?;
    let cutoff = (Utc::now() - ChronoDuration::days(days)).to_rfc3339();
    let mut transaction = db.begin().await?;
    sqlx::query("DELETE FROM packet_audit WHERE packet_id IN (SELECT id FROM packets WHERE team_id=? AND created_at < ?)")
        .bind(team_id)
        .bind(&cutoff)
        .execute(&mut *transaction)
        .await?;
    sqlx::query("DELETE FROM packets WHERE team_id=? AND created_at < ?")
        .bind(team_id)
        .bind(cutoff)
        .execute(&mut *transaction)
        .await?;
    transaction.commit().await?;
    Ok(())
}

async fn purge_expired_transient_rows(db: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM sessions WHERE expires_at <= ?")
        .bind(Utc::now().to_rfc3339())
        .execute(db)
        .await?;
    sqlx::query("DELETE FROM oauth_states WHERE created_at < ?")
        .bind((Utc::now() - ChronoDuration::minutes(10)).to_rfc3339())
        .execute(db)
        .await?;
    Ok(())
}

async fn purge_all_expired_data(db: &SqlitePool) -> Result<(), sqlx::Error> {
    purge_expired_transient_rows(db).await?;
    let teams = sqlx::query_scalar::<_, String>("SELECT id FROM teams")
        .fetch_all(db)
        .await?;
    for team_id in teams {
        purge_team_packets(db, &team_id).await?;
    }
    Ok(())
}
fn evidence_is_complete(data: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(data)
        .ok()
        .and_then(|value| {
            value
                .get("checks")
                .and_then(|checks| checks.as_array())
                .cloned()
        })
        .map(|checks| {
            !checks.is_empty()
                && checks.iter().all(|check| {
                    matches!(
                        check.get("state").and_then(|value| value.as_str()),
                        Some("ready") | Some("done")
                    )
                })
        })
        .unwrap_or(false)
}
async fn update_packet_evidence(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<EvidenceUpdate>,
) -> Result<Json<Packet>, AppError> {
    let who = session(&state, &headers).await?;
    let packet = sqlx::query_as::<_, Packet>("SELECT id,title,owner,status,data,created_at,approved_by,approved_at,source_url FROM packets WHERE id=? AND team_id=?").bind(&id).bind(&who.team_id).fetch_optional(&state.db).await?.ok_or_else(|| AppError::not_found("That review packet was not found in this team."))?;
    if packet.status == "approved" {
        return Err(AppError::conflict(
            "Approved packets are immutable. Create a new review packet for a changed decision.",
        ));
    }
    if !input
        .data
        .get("checks")
        .is_some_and(|checks| checks.as_array().is_some())
    {
        return Err(AppError::bad(
            "Review evidence must include the packet checks.",
        ));
    }
    let data = input.data.to_string();
    sqlx::query("UPDATE packets SET data=? WHERE id=? AND team_id=?")
        .bind(data)
        .bind(&id)
        .bind(&who.team_id)
        .execute(&state.db)
        .await?;
    audit(
        &state.db,
        &id,
        &who.login,
        "evidence_updated",
        "Review evidence was saved.",
    )
    .await?;
    get_packet(State(state), headers, Path(id)).await
}
async fn approve_packet(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<Approval>,
) -> Result<Json<Packet>, AppError> {
    let who = session(&state, &headers).await?;
    let packet = sqlx::query_as::<_, Packet>("SELECT id,title,owner,status,data,created_at,approved_by,approved_at,source_url FROM packets WHERE id=? AND team_id=?").bind(&id).bind(&who.team_id).fetch_optional(&state.db).await?.ok_or_else(|| AppError::not_found("That review packet was not found in this team."))?;
    if packet.owner != who.login {
        return Err(AppError::forbidden(
            "Only the named owner can approve this review packet.",
        ));
    }
    if packet.status == "approved" {
        return Err(AppError::conflict(
            "This packet is already approved and its approval is immutable.",
        ));
    }
    if !evidence_is_complete(&packet.data) {
        return Err(AppError::bad(
            "Resolve and save every review check before approval.",
        ));
    }
    let updated = sqlx::query(
        "UPDATE packets SET status='approved',approved_by=?,approved_at=? WHERE id=? AND team_id=? AND status!='approved'",
    )
    .bind(&who.login)
    .bind(Utc::now().to_rfc3339())
    .bind(&id)
    .bind(&who.team_id)
    .execute(&state.db)
    .await?
    .rows_affected();
    if updated != 1 {
        let current: Option<String> =
            sqlx::query_scalar("SELECT status FROM packets WHERE id=? AND team_id=?")
                .bind(&id)
                .bind(&who.team_id)
                .fetch_optional(&state.db)
                .await?;
        return Err(match current.as_deref() {
            Some("approved") => {
                AppError::conflict("This packet is already approved and its approval is immutable.")
            }
            _ => AppError::not_found("That review packet was not found in this team."),
        });
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
async fn installation_token(state: &AppState, team_id: &str) -> Result<String, AppError> {
    if !state.github.app_ready() {
        return Err(AppError::service_unavailable(
            "Connect the Diff Gate GitHub App before importing a pull request.",
        ));
    }
    let installation_id = state.github.installation_for(team_id).ok_or_else(|| AppError::forbidden("No GitHub App installation is bound to this Sociobot team. Ask a team administrator to install and bind it."))?;
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
        installation_id
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
    let token = installation_token(&state, &who.team_id).await?;
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
    let changed = github_changed_files(&state, &base, &token).await?;
    let (has_contract, has_migration) = classify_changed_paths(&changed);
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

fn classify_changed_paths(changed: &[String]) -> (bool, bool) {
    let has_migration = changed
        .iter()
        .any(|f| f.contains("migration") || f.starts_with("db/"));
    let has_contract = changed
        .iter()
        .any(|f| f.contains("api/") || f.contains("contract") || f.ends_with(".graphql"));
    (has_contract, has_migration)
}

async fn github_changed_files(
    state: &AppState,
    base: &str,
    token: &str,
) -> Result<Vec<String>, AppError> {
    let mut page = 1;
    let mut changed = Vec::new();
    loop {
        let response = state
            .http
            .get(format!("{base}/files?per_page=100&page={page}"))
            .header("user-agent", "diff-gate")
            .header("accept", "application/vnd.github+json")
            .bearer_auth(token)
            .send()
            .await
            .map_err(|_| AppError::service_unavailable("GitHub could not load changed files."))?
            .error_for_status()
            .map_err(|_| AppError::service_unavailable("GitHub could not load changed files."))?;
        let files = response
            .json::<Vec<GithubFile>>()
            .await
            .map_err(|_| AppError::service_unavailable("GitHub returned invalid changed files."))?;
        let count = files.len();
        changed.extend(files.into_iter().map(|file| file.filename));
        if count < 100 {
            break;
        }
        page += 1;
        if page > 100 {
            return Err(AppError::bad("This pull request has more than 10,000 changed files and cannot be imported safely."));
        }
    }
    Ok(changed)
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
    let value = if path.starts_with("/assets/") {
        "public, max-age=31536000, immutable"
    } else if path.ends_with(".webp") || path.ends_with(".png") || path.ends_with(".svg") {
        "public, max-age=3600, must-revalidate"
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
    fn forbidden(message: &str) -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            message: message.into(),
        }
    }
    fn conflict(message: &str) -> Self {
        Self {
            status: StatusCode::CONFLICT,
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
        .route(
            "/api/packets/:id",
            get(get_packet)
                .put(update_packet_evidence)
                .delete(delete_packet),
        )
        .route("/api/packets/:id/approve", post(approve_packet))
        .route("/api/packets/:id/audit", get(list_packet_audit))
        .route("/api/settings", get(get_settings).put(update_settings))
        .route("/api/github/import", post(import_github_pr))
        .route("/api/auth/status", get(auth_status))
        .route("/auth/entra", get(entra_login))
        .route("/auth/entra/callback", get(entra_callback))
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
    headers.insert(
        HeaderName::from_static("strict-transport-security"),
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
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
    sqlx::query("CREATE TABLE IF NOT EXISTS team_settings (team_id TEXT PRIMARY KEY,retention_days INTEGER NOT NULL CHECK(retention_days BETWEEN 1 AND 3650))").execute(db).await?;
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
    let identity = EntraConfig::from_env();
    let github = GithubConfig::from_env();
    info!(
        database_config,
        entra_identity = identity.ready(),
        github_app = github.app_ready(),
        "Diff Gate starting; only optional GitHub configuration was supplied"
    );
    let db = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;
    create_schema(&db).await?;
    purge_all_expired_data(&db).await?;
    let state = AppState {
        db,
        build,
        limits: Arc::new(Mutex::new(HashMap::new())),
        identity,
        github,
        http: Client::new(),
    };
    let cleanup_db = state.db.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(3600));
        loop {
            interval.tick().await;
            if let Err(error) = purge_all_expired_data(&cleanup_db).await {
                tracing::warn!(%error, "scheduled retention cleanup failed");
            }
        }
    });
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
            identity: EntraConfig::default(),
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
        sqlx::query("INSERT INTO packets (id,team_id,title,owner,status,data,created_at) VALUES ('packet-b','b','Private change','bea','needs review','{\"checks\":[{\"state\":\"done\"}]}',?)")
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
    async fn approval_rejects_missing_evidence_and_wrong_owner_and_persists_saved_evidence() {
        let (app, db) = test_app().await;
        let now = (Utc::now() + ChronoDuration::days(1)).to_rfc3339();
        sqlx::query("INSERT INTO teams (id,name,created_at) VALUES ('team','Team',?)")
            .bind(&now)
            .execute(&db)
            .await
            .unwrap();
        sqlx::query("INSERT INTO sessions (token,team_id,login,expires_at) VALUES ('owner-token','team','owner',?),('reviewer-token','team','reviewer',?)")
            .bind(&now).bind(&now).execute(&db).await.unwrap();
        sqlx::query("INSERT INTO packets (id,team_id,title,owner,status,data,created_at) VALUES ('packet','team','Protected change','owner','needs review','{\"checks\":[{\"state\":\"missing\"}]}',?)")
            .bind(&now).execute(&db).await.unwrap();
        let missing = app
            .clone()
            .oneshot(
                Request::post("/api/packets/packet/approve")
                    .header("cookie", "diff_gate_session=owner-token")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(missing.status(), StatusCode::BAD_REQUEST);
        let evidence = app
            .clone()
            .oneshot(
                Request::put("/api/packets/packet")
                    .header("cookie", "diff_gate_session=owner-token")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        "{\"data\":{\"checks\":[{\"state\":\"done\"}]}}",
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(evidence.status(), StatusCode::OK);
        let wrong_owner = app
            .clone()
            .oneshot(
                Request::post("/api/packets/packet/approve")
                    .header("cookie", "diff_gate_session=reviewer-token")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(wrong_owner.status(), StatusCode::FORBIDDEN);
        let approved = app
            .oneshot(
                Request::post("/api/packets/packet/approve")
                    .header("cookie", "diff_gate_session=owner-token")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(approved.status(), StatusCode::OK);
        let state: String = sqlx::query_scalar("SELECT data FROM packets WHERE id='packet'")
            .fetch_one(&db)
            .await
            .unwrap();
        assert!(state.contains("done"));
        let audit_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM packet_audit WHERE packet_id='packet' AND action='evidence_updated'").fetch_one(&db).await.unwrap();
        assert_eq!(audit_count, 1);
    }
    #[test]
    fn entra_and_github_installations_are_configured_per_team() {
        let identity = EntraConfig {
            authority: Some("https://sociobotcustomers.ciamlogin.com/tenant".into()),
            client_id: Some("client-id".into()),
            client_secret: Some("secret".into()),
            public_base: "https://agent-diff-gate.sociobot.in".into(),
            team_claim: "extension_DiffGateTeam".into(),
        };
        assert!(identity.ready());
        let wrong_tenant = EntraConfig {
            authority: Some("https://login.microsoftonline.com/tenant".into()),
            ..identity.clone()
        };
        assert!(!wrong_tenant.ready());
        assert!(
            sociobot_entra_authority("http://sociobotcustomers.ciamlogin.com/tenant").is_none()
        );
        assert!(
            sociobot_entra_authority("https://sociobotcustomers.ciamlogin.com:8443/tenant")
                .is_none()
        );
        assert!(sociobot_entra_authority("https://evil.example/tenant").is_none());
        assert_eq!(
            identity.callback_url(),
            "https://agent-diff-gate.sociobot.in/auth/entra/callback"
        );
        let github = GithubConfig {
            app_id: Some("app".into()),
            private_key: Some("key".into()),
            app_slug: None,
            installations: HashMap::from([(String::from("entra:team-a"), String::from("101"))]),
        };
        assert_eq!(
            github.installation_for("entra:team-a").as_deref(),
            Some("101")
        );
        assert_eq!(github.installation_for("entra:team-b"), None);
        assert!(github.app_ready());
    }
    #[tokio::test]
    async fn github_import_paginates_and_classifies_all_changed_paths() {
        async fn files(
            axum::extract::Query(query): axum::extract::Query<HashMap<String, String>>,
        ) -> Json<Vec<serde_json::Value>> {
            if query.get("page").map(String::as_str) == Some("1") {
                return Json(
                    (0..100)
                        .map(|index| serde_json::json!({"filename":format!("src/file-{index}.ts")}))
                        .collect(),
                );
            }
            Json(vec![
                serde_json::json!({"filename":"src/api/contracts/user.graphql"}),
                serde_json::json!({"filename":"db/migrations/20260828_users.sql"}),
            ])
        }
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(listener, Router::new().route("/pulls/42/files", get(files)))
                .await
                .unwrap();
        });
        let (_, db) = test_app().await;
        let state = AppState {
            db,
            build: "fixture".into(),
            limits: Arc::new(Mutex::new(HashMap::new())),
            identity: EntraConfig::default(),
            github: GithubConfig::default(),
            http: Client::new(),
        };
        let changed = github_changed_files(
            &state,
            &format!("http://{address}/pulls/42"),
            "fixture-installation-token",
        )
        .await
        .unwrap();
        assert_eq!(changed.len(), 102);
        assert_eq!(classify_changed_paths(&changed), (true, true));
        server.abort();
    }
    #[tokio::test]
    async fn retention_and_explicit_deletion_remove_packets_and_audit() {
        let (app, db) = test_app().await;
        let future = (Utc::now() + ChronoDuration::days(1)).to_rfc3339();
        let old = (Utc::now() - ChronoDuration::days(31)).to_rfc3339();
        sqlx::query("INSERT INTO teams (id,name,created_at) VALUES ('team','Team',?)")
            .bind(&future)
            .execute(&db)
            .await
            .unwrap();
        sqlx::query("INSERT INTO sessions (token,team_id,login,expires_at) VALUES ('token','team','owner',?)")
            .bind(&future).execute(&db).await.unwrap();
        sqlx::query("INSERT INTO packets (id,team_id,title,owner,status,data,created_at) VALUES ('old','team','Old','owner','needs review','{}',?),('current','team','Current','owner','needs review','{}',?)")
            .bind(&old).bind(&future).execute(&db).await.unwrap();
        sqlx::query("INSERT INTO packet_audit (id,packet_id,actor,action,detail,created_at) VALUES ('old-audit','old','owner','created','Old',?),('current-audit','current','owner','created','Current',?)")
            .bind(&old).bind(&future).execute(&db).await.unwrap();
        let defaults = app
            .clone()
            .oneshot(
                Request::get("/api/settings")
                    .header("cookie", "diff_gate_session=token")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let defaults_body = to_bytes(defaults.into_body(), usize::MAX).await.unwrap();
        let defaults_json: serde_json::Value = serde_json::from_slice(&defaults_body).unwrap();
        assert_eq!(defaults_json["retention_days"], DEFAULT_RETENTION_DAYS);
        let settings = app
            .clone()
            .oneshot(
                Request::put("/api/settings")
                    .header("cookie", "diff_gate_session=token")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from("{\"retention_days\":30}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(settings.status(), StatusCode::OK);
        let old_packets: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM packets WHERE id='old'")
            .fetch_one(&db)
            .await
            .unwrap();
        let old_audits: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM packet_audit WHERE packet_id='old'")
                .fetch_one(&db)
                .await
                .unwrap();
        assert_eq!((old_packets, old_audits), (0, 0));
        let deleted = app
            .oneshot(
                Request::delete("/api/packets/current")
                    .header("cookie", "diff_gate_session=token")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(deleted.status(), StatusCode::NO_CONTENT);
        let remaining: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM packet_audit WHERE packet_id='current'")
                .fetch_one(&db)
                .await
                .unwrap();
        assert_eq!(remaining, 0);
    }
    #[tokio::test]
    async fn audit_history_is_team_scoped_and_concurrent_approval_reports_conflict() {
        let (app, db) = test_app().await;
        let future = (Utc::now() + ChronoDuration::days(1)).to_rfc3339();
        sqlx::query(
            "INSERT INTO teams (id,name,created_at) VALUES ('team','Team',?),('other','Other',?)",
        )
        .bind(&future)
        .bind(&future)
        .execute(&db)
        .await
        .unwrap();
        sqlx::query("INSERT INTO sessions (token,team_id,login,expires_at) VALUES ('token','team','owner',?),('other-token','other','owner',?)")
            .bind(&future).bind(&future).execute(&db).await.unwrap();
        sqlx::query("INSERT INTO packets (id,team_id,title,owner,status,data,created_at) VALUES ('packet','team','Ready','owner','needs review','{\"checks\":[{\"state\":\"done\"}]}',?)")
            .bind(&future).execute(&db).await.unwrap();
        let request = || {
            Request::post("/api/packets/packet/approve")
                .header("cookie", "diff_gate_session=token")
                .header("content-type", "application/json")
                .body(axum::body::Body::from("{}"))
                .unwrap()
        };
        let (first, second) = tokio::join!(
            app.clone().oneshot(request()),
            app.clone().oneshot(request())
        );
        let statuses = [first.unwrap().status(), second.unwrap().status()];
        assert!(statuses.contains(&StatusCode::OK));
        assert!(statuses.contains(&StatusCode::CONFLICT));
        let audit = app
            .clone()
            .oneshot(
                Request::get("/api/packets/packet/audit")
                    .header("cookie", "diff_gate_session=token")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(audit.status(), StatusCode::OK);
        let hidden = app
            .oneshot(
                Request::get("/api/packets/packet/audit")
                    .header("cookie", "diff_gate_session=other-token")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(hidden.status(), StatusCode::NOT_FOUND);
    }
    #[tokio::test]
    async fn response_policy_distinguishes_hashed_and_stable_assets() {
        let (app, _) = test_app().await;
        let health = app
            .clone()
            .oneshot(
                Request::get("/health")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            health.headers()["strict-transport-security"],
            "max-age=31536000; includeSubDomains"
        );
        let stable = app
            .oneshot(
                Request::get("/change-control.webp")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            stable.headers()[header::CACHE_CONTROL],
            "public, max-age=3600, must-revalidate"
        );
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
