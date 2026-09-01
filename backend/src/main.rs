use axum::{
    extract::{rejection::JsonRejection, FromRequest, Path, Request, State},
    http::{header, HeaderMap, HeaderName, HeaderValue, StatusCode},
    middleware,
    response::{IntoResponse, Redirect, Response},
    routing::{get, get_service, post},
    Json, Router,
};
use base64::Engine as _;
use chrono::{Duration as ChronoDuration, Utc};
use jsonwebtoken::{
    decode, decode_header, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation,
};
use reqwest::Client;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{sqlite::SqlitePoolOptions, FromRow, SqlitePool};
use std::{
    collections::{BTreeSet, HashMap},
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
const DURABLE_DATABASE_URL: &str = "sqlite:/data/diff-gate.db?mode=rwc&vfs=unix-none";
const DEPLOYMENT_CONFIG_VERSION: &str = "6";
const PRODUCT_PUBLIC_BASE_URL: &str = "https://agent-diff-gate.sociobot.in";
const SOCIOBOT_TENANT_ID: &str = "35c6fe40-0ec0-46b6-98c6-213ad4de6650";
const SOCIOBOT_CLIENT_ID: &str = "25c704f4-465a-47af-80ab-2c489466b697";
const SOCIOBOT_AUTHORITY: &str =
    "https://sociobotcustomers.ciamlogin.com/35c6fe40-0ec0-46b6-98c6-213ad4de6650";

#[derive(Clone)]
struct AppState {
    db: SqlitePool,
    build: String,
    storage_id: String,
    stateful_production_ready: bool,
    limits: Arc<Mutex<HashMap<String, Window>>>,
    identity: EntraConfig,
    github: GithubConfig,
    github_api_base: String,
    http: Client,
}
fn supplied_stateful_production_contract() -> bool {
    [
        ("DATABASE_URL", DURABLE_DATABASE_URL),
        ("PUBLIC_BASE_URL", PRODUCT_PUBLIC_BASE_URL),
        ("ENTRA_AUTHORITY", SOCIOBOT_AUTHORITY),
        ("ENTRA_TENANT_ID", SOCIOBOT_TENANT_ID),
        ("ENTRA_CLIENT_ID", SOCIOBOT_CLIENT_ID),
        ("ENTRA_TEAM_CLAIM", "oid"),
        ("DEPLOYMENT_CONFIG_VERSION", DEPLOYMENT_CONFIG_VERSION),
    ]
    .iter()
    .all(|(name, expected)| env::var(name).ok().as_deref() == Some(*expected))
}
fn is_production_host(headers: &HeaderMap) -> bool {
    headers
        .get(header::HOST)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.split_once(':').map_or(value, |(host, _)| host))
        .is_some_and(|host| host.eq_ignore_ascii_case("agent-diff-gate.sociobot.in"))
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
    tenant_id: String,
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
        let tenant_id = env::var("ENTRA_TENANT_ID").unwrap_or_else(|_| SOCIOBOT_TENANT_ID.into());
        let authority = env::var("ENTRA_AUTHORITY").unwrap_or_else(|_| SOCIOBOT_AUTHORITY.into());
        Self {
            authority: sociobot_entra_authority(&authority),
            client_id: Some(
                env::var("ENTRA_CLIENT_ID").unwrap_or_else(|_| SOCIOBOT_CLIENT_ID.into()),
            ),
            tenant_id,
            // A missing deployment setting must never turn a real Entra sign-in into a
            // localhost redirect. Local developers can still explicitly use localhost.
            public_base: configured_public_base(env::var("PUBLIC_BASE_URL").ok().as_deref()),
            team_claim: env::var("ENTRA_TEAM_CLAIM").unwrap_or_else(|_| "oid".into()),
        }
    }
    fn ready(&self) -> bool {
        self.authority
            .as_deref()
            .and_then(sociobot_entra_authority)
            .is_some_and(|authority| authority.ends_with(&format!("/{}", self.tenant_id)))
            && self.tenant_id == SOCIOBOT_TENANT_ID
            && self.client_id.is_some()
    }
    fn callback_url(&self) -> String {
        format!("{}/auth/callback", self.public_base.trim_end_matches('/'))
    }
}
fn configured_public_base(value: Option<&str>) -> String {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(PRODUCT_PUBLIC_BASE_URL)
        .to_string()
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
    storage_id: String,
}
#[derive(Serialize)]
struct AuthStatus {
    service_ready: bool,
    authenticated: bool,
    entra_sign_in_configured: bool,
    github_app_setup_available: bool,
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
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct PolicyRule {
    path: String,
    required_owner: String,
}
#[derive(Serialize, Clone)]
struct RepositoryPolicy {
    repository: String,
    rules: Vec<PolicyRule>,
}
#[derive(Deserialize)]
struct RepositoryPolicyUpdate {
    repository: String,
    rules: Vec<PolicyRule>,
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
    test_evidence: Option<TestEvidenceInput>,
}
#[derive(Deserialize)]
struct Approval {
    note: Option<String>,
}
#[derive(Deserialize)]
struct EvidenceUpdate {
    data: serde_json::Value,
    test_evidence: Option<TestEvidenceInput>,
}
#[derive(Deserialize)]
struct TestEvidenceInput {
    command: String,
    result: String,
}
#[derive(Deserialize)]
struct ImportRequest {
    pr_url: String,
}

/// JSON extraction is part of the API boundary. Axum's default rejection text
/// includes implementation wording, so convert it into one stable product
/// error before any handler can return a response.
struct AppJson<T>(T);

#[axum::async_trait]
impl<S, T> FromRequest<S> for AppJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Send,
{
    type Rejection = AppError;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        Json::<T>::from_request(request, state)
            .await
            .map(|Json(value)| Self(value))
            .map_err(|rejection| AppError::invalid_json_input(&json_rejection_details(&rejection)))
    }
}

fn json_rejection_details(rejection: &JsonRejection) -> String {
    let mut details = rejection.to_string();
    let mut source = std::error::Error::source(rejection);
    while let Some(error) = source {
        details.push(' ');
        details.push_str(&error.to_string());
        source = error.source();
    }
    details
}
#[derive(Deserialize)]
struct GithubPull {
    title: String,
    html_url: String,
    user: GithubPullUser,
    head: GithubHead,
}
#[derive(Deserialize)]
struct GithubHead {
    sha: String,
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
    tid: String,
    nonce: Option<String>,
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
#[derive(FromRow, Clone)]
struct TeamGithubApp {
    app_id: String,
    private_key: String,
    app_slug: String,
    installation_id: Option<String>,
}

async fn team_github_app(
    db: &SqlitePool,
    team_id: &str,
) -> Result<Option<TeamGithubApp>, sqlx::Error> {
    sqlx::query_as(
        "SELECT app_id,private_key,app_slug,installation_id FROM github_team_apps WHERE team_id=?",
    )
    .bind(team_id)
    .fetch_optional(db)
    .await
}

fn github_install_url(slug: &str) -> String {
    format!("https://github.com/apps/{slug}/installations/new")
}

async fn validate_entra_token(
    state: &AppState,
    id_token: &str,
    expected_nonce: &str,
) -> Result<EntraClaims, AppError> {
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
    if header.alg != Algorithm::RS256 {
        return Err(AppError::unauthorized(
            "Sociobot identity tokens must use RS256.",
        ));
    }
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
    let claims = decode::<EntraClaims>(id_token, &key, &validation)
        .map(|data| data.claims)
        .map_err(|_| AppError::unauthorized("Sociobot identity token validation failed."))?;
    if claims.tid != state.identity.tenant_id || claims.nonce.as_deref() != Some(expected_nonce) {
        return Err(AppError::unauthorized(
            "Sociobot identity token tenant or sign-in nonce did not match.",
        ));
    }
    Ok(claims)
}

async fn health(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let safe = state.stateful_production_ready || !is_production_host(&headers);
    (
        if safe {
            StatusCode::OK
        } else {
            StatusCode::SERVICE_UNAVAILABLE
        },
        Json(Health {
            status: if safe { "ok" } else { "unsafe_configuration" },
            build: state.build,
            storage_id: state.storage_id,
        }),
    )
        .into_response()
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
async fn auth_status(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<AuthStatus>, AppError> {
    // This is the only API read that the public landing page makes. When a
    // generic deployment loses the required SQLite topology, keep every
    // stateful route fail-closed but answer this probe without touching the
    // ephemeral database. That lets the page truthfully show its recovery
    // state instead of generating a cold-load 503 and console error.
    let service_ready = state.stateful_production_ready || !is_production_host(&headers);
    if !service_ready {
        return Ok(Json(AuthStatus {
            service_ready: false,
            authenticated: false,
            entra_sign_in_configured: false,
            github_app_setup_available: false,
            github_app_configured: false,
            install_url: None,
            user: None,
            team: None,
        }));
    }
    let _ = purge_expired_transient_rows(&state.db).await;
    let active = session(&state, &headers).await.ok();
    let team_app = if let Some(active) = active.as_ref() {
        team_github_app(&state.db, &active.team_id).await?
    } else {
        None
    };
    let team_install_url = team_app
        .as_ref()
        .map(|app| github_install_url(&app.app_slug));
    Ok(Json(AuthStatus {
        service_ready: true,
        authenticated: active.is_some(),
        entra_sign_in_configured: state.identity.ready(),
        github_app_setup_available: true,
        github_app_configured: team_app
            .as_ref()
            .and_then(|app| app.installation_id.as_ref())
            .is_some()
            || (active
                .as_ref()
                .and_then(|session| state.github.installation_for(&session.team_id))
                .is_some()
                && state.github.app_ready()),
        install_url: team_install_url.or_else(|| {
            state
                .github
                .app_slug
                .as_ref()
                .map(|slug| github_install_url(slug))
        }),
        user: active.as_ref().map(|s| s.login.clone()),
        team: active.map(|s| s.team_name),
    }))
}
async fn entra_login(State(state): State<AppState>) -> Result<Redirect, AppError> {
    if !state.identity.ready() {
        return Err(AppError::service_unavailable(
            "Sociobot Entra sign-in is not configured on this deployment.",
        ));
    }
    purge_expired_transient_rows(&state.db).await?;
    let oauth_state = Uuid::new_v4().to_string();
    let nonce = Uuid::new_v4().to_string();
    let verifier = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    let challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(Sha256::digest(verifier.as_bytes()));
    sqlx::query("INSERT INTO oauth_pkce (state,nonce,verifier,created_at) VALUES (?,?,?,?)")
        .bind(&oauth_state)
        .bind(&nonce)
        .bind(&verifier)
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
        .append_pair("state", &oauth_state)
        .append_pair("nonce", &nonce)
        .append_pair("code_challenge", &challenge)
        .append_pair("code_challenge_method", "S256");
    Ok(Redirect::temporary(url.as_str()))
}
#[derive(Deserialize)]
struct OAuthCallback {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}
fn oauth_error_message(error: Option<&str>, description: Option<&str>) -> &'static str {
    let description = description.unwrap_or_default().to_ascii_lowercase();
    match error.unwrap_or_default() {
        "access_denied" => "Sign-in was cancelled or your account did not grant access.",
        "temporarily_unavailable" | "server_error" => {
            "Sociobot could not complete sign-in right now."
        }
        "interaction_required" | "login_required" | "consent_required" => {
            "Sociobot needs you to sign in again."
        }
        _ if description.contains("cancel") || description.contains("denied") => {
            "Sign-in was cancelled or your account did not grant access."
        }
        _ => "Sociobot did not complete sign-in.",
    }
}
fn oauth_error_page(message: &str) -> Response {
    let body = format!(
        r##"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <meta name="theme-color" content="#17212B">
    <meta name="description" content="Return to Diff Gate after a Sociobot sign-in problem.">
    <meta name="robots" content="noindex">
    <link rel="canonical" href="https://agent-diff-gate.sociobot.in/auth/callback">
    <link rel="icon" href="/favicon.svg" type="image/svg+xml">
    <link rel="apple-touch-icon" href="/apple-touch-icon.png">
    <meta property="og:title" content="Sign-in did not complete — Diff Gate">
    <meta property="og:description" content="Return to Diff Gate after a Sociobot sign-in problem.">
    <meta property="og:image" content="https://agent-diff-gate.sociobot.in/social.webp">
    <meta name="twitter:card" content="summary_large_image">
    <meta name="twitter:title" content="Sign-in did not complete — Diff Gate">
    <meta name="twitter:description" content="Return to Diff Gate after a Sociobot sign-in problem.">
    <meta name="twitter:image" content="https://agent-diff-gate.sociobot.in/social.webp">
    <link rel="stylesheet" href="/404.css">
    <title>Sign-in did not complete — Diff Gate</title>
  </head>
  <body>
    <a class="skip" href="#main">Skip to content</a>
    <header>
      <a class="wordmark" href="/">≡ Diff Gate</a>
      <nav aria-label="Main navigation"><a href="/?demo=1">Demo</a><a href="/#how">How it works</a><a href="/privacy">Privacy</a></nav>
    </header>
    <main id="main">
      <p class="eyebrow">Sign-in error</p>
      <h1>Sign-in did not complete</h1>
      <p>{message}</p>
      <p>No review data was changed. Try again, return to Diff Gate, or open the sample.</p>
      <div class="actions">
        <a class="action" href="/auth/entra">Try sign-in again</a>
        <a class="secondary-action" href="/">Return to Diff Gate</a>
        <a class="secondary-action" href="/?demo=1">Try it with sample data</a>
      </div>
    </main>
    <footer><span>Review agent-authored changes before merge.</span><span><a href="/privacy">Privacy</a><a href="/terms">Terms</a><span>Built by Param Factory</span></span><small>v0.5.0</small></footer>
  </body>
</html>"##
    );
    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "text/html; charset=utf-8"),
            (header::HeaderName::from_static("x-robots-tag"), "noindex"),
        ],
        body,
    )
        .into_response()
}
async fn entra_callback(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<OAuthCallback>,
) -> Result<Response, AppError> {
    if query.error.is_some() || query.error_description.is_some() {
        if let Some(oauth_state) = query.state.as_deref().filter(|value| !value.is_empty()) {
            let _ = sqlx::query("DELETE FROM oauth_pkce WHERE state=?")
                .bind(oauth_state)
                .execute(&state.db)
                .await;
        }
        return Ok(oauth_error_page(oauth_error_message(
            query.error.as_deref(),
            query.error_description.as_deref(),
        )));
    }
    if !state.identity.ready() {
        return Err(AppError::service_unavailable(
            "Sociobot Entra sign-in is not configured on this deployment.",
        ));
    }
    let Some(code) = query.code.as_deref().filter(|value| !value.is_empty()) else {
        return Ok(oauth_error_page(
            "Sociobot returned without the sign-in details Diff Gate needs.",
        ));
    };
    let Some(oauth_state) = query.state.as_deref().filter(|value| !value.is_empty()) else {
        return Ok(oauth_error_page(
            "That Sociobot sign-in link is incomplete or expired.",
        ));
    };
    let saved: Option<(String, String)> =
        sqlx::query_as("SELECT nonce,verifier FROM oauth_pkce WHERE state=?")
            .bind(oauth_state)
            .fetch_optional(&state.db)
            .await?;
    let Some((nonce, verifier)) = saved else {
        return Err(AppError::bad(
            "That Sociobot sign-in link expired. Start again.",
        ));
    };
    sqlx::query("DELETE FROM oauth_pkce WHERE state=?")
        .bind(oauth_state)
        .execute(&state.db)
        .await?;
    let authority = state.identity.authority.as_deref().unwrap_or_default();
    let token = state
        .http
        .post(format!("{authority}/oauth2/v2.0/token"))
        .header("origin", &state.identity.public_base)
        .form(&[
            (
                "client_id",
                state.identity.client_id.as_deref().unwrap_or_default(),
            ),
            ("code", code),
            ("redirect_uri", &state.identity.callback_url()),
            ("grant_type", "authorization_code"),
            ("code_verifier", &verifier),
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
    let claims = validate_entra_token(&state, id_token, &nonce).await?;
    let team_value = if state.identity.team_claim == "oid" {
        claims.oid.clone()
    } else {
        claims.extra.get(&state.identity.team_claim).and_then(|value| match value { serde_json::Value::String(value) => Some(value.clone()), serde_json::Value::Array(values) => values.first().and_then(|v| v.as_str()).map(str::to_string), _ => None })
    }.filter(|value| !value.trim().is_empty()).ok_or_else(|| AppError::forbidden("Your Sociobot account has no Diff Gate team claim. Ask a team administrator to assign one."))?;
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
    AppJson(input): AppJson<NewPacket>,
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
    let data =
        normalize_packet_evidence(input.data, input.test_evidence.as_ref(), &who.login, None)?;
    let packet = Packet {
        id: Uuid::new_v4().to_string(),
        title: input.title.trim().to_string(),
        owner,
        status: "needs review".into(),
        data: data.to_string(),
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
    AppJson(input): AppJson<SettingsUpdate>,
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

fn normalized_repository(raw: &str) -> Option<String> {
    let value = raw.trim().to_ascii_lowercase();
    let (owner, repo) = value.split_once('/')?;
    let valid_part = |part: &str| {
        !part.is_empty()
            && part.len() <= 100
            && part
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
    };
    (valid_part(owner) && valid_part(repo) && !repo.contains('/')).then_some(value)
}

fn validate_policy_rules(rules: &[PolicyRule]) -> Result<(), AppError> {
    if rules.is_empty() || rules.len() > 20 {
        return Err(AppError::bad("Add between 1 and 20 sensitive path rules."));
    }
    for rule in rules {
        let path = rule.path.trim();
        if path.is_empty()
            || path.len() > 200
            || path.starts_with('/')
            || path.contains("..")
            || rule.required_owner.trim().is_empty()
            || rule.required_owner.trim().len() > 180
        {
            return Err(AppError::bad(
                "Each policy rule needs a relative path and a required owner.",
            ));
        }
    }
    Ok(())
}

async fn list_repository_policies(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<RepositoryPolicy>>, AppError> {
    let who = session(&state, &headers).await?;
    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT repository,rules FROM repository_policies WHERE team_id=? ORDER BY repository",
    )
    .bind(&who.team_id)
    .fetch_all(&state.db)
    .await?;
    let policies = rows
        .into_iter()
        .filter_map(|(repository, rules)| {
            serde_json::from_str(&rules)
                .ok()
                .map(|rules| RepositoryPolicy { repository, rules })
        })
        .collect();
    Ok(Json(policies))
}

async fn save_repository_policy(
    State(state): State<AppState>,
    headers: HeaderMap,
    AppJson(input): AppJson<RepositoryPolicyUpdate>,
) -> Result<Json<RepositoryPolicy>, AppError> {
    let who = session(&state, &headers).await?;
    let repository = normalized_repository(&input.repository)
        .ok_or_else(|| AppError::bad("Use a GitHub repository shaped owner/repository."))?;
    validate_policy_rules(&input.rules)?;
    let rules: Vec<PolicyRule> = input
        .rules
        .iter()
        .map(|rule| PolicyRule {
            path: rule.path.trim().to_string(),
            required_owner: rule.required_owner.trim().to_string(),
        })
        .collect();
    let policy = RepositoryPolicy { repository, rules };
    sqlx::query("INSERT INTO repository_policies (team_id,repository,rules,updated_at) VALUES (?,?,?,?) ON CONFLICT(team_id,repository) DO UPDATE SET rules=excluded.rules,updated_at=excluded.updated_at")
        .bind(&who.team_id)
        .bind(&policy.repository)
        .bind(serde_json::to_string(&policy.rules).expect("policy rules serialize"))
        .bind(Utc::now().to_rfc3339())
        .execute(&state.db)
        .await?;
    Ok(Json(policy))
}

async fn repository_policy(
    db: &SqlitePool,
    team_id: &str,
    repository: &str,
) -> Result<Option<RepositoryPolicy>, AppError> {
    let row: Option<String> = sqlx::query_scalar(
        "SELECT rules FROM repository_policies WHERE team_id=? AND repository=?",
    )
    .bind(team_id)
    .bind(repository)
    .fetch_optional(db)
    .await?;
    row.map(|rules| {
        serde_json::from_str(&rules)
            .map(|rules| RepositoryPolicy {
                repository: repository.to_string(),
                rules,
            })
            .map_err(|_| {
                AppError::service_unavailable("This repository policy is invalid. Save it again.")
            })
    })
    .transpose()
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
    sqlx::query("DELETE FROM oauth_pkce WHERE created_at < ?")
        .bind((Utc::now() - ChronoDuration::minutes(10)).to_rfc3339())
        .execute(db)
        .await?;
    sqlx::query("DELETE FROM github_app_states WHERE created_at < ?")
        .bind((Utc::now() - ChronoDuration::minutes(20)).to_rfc3339())
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
fn valid_recorded_evidence(value: &serde_json::Value) -> bool {
    let present = |field: &str, minimum: usize| {
        value
            .get(field)
            .and_then(|item| item.as_str())
            .is_some_and(|item| item.trim().len() >= minimum)
    };
    present("command", 3)
        && present("result", 2)
        && present("recorded_by", 1)
        && value
            .get("recorded_at")
            .and_then(|item| item.as_str())
            .and_then(|item| chrono::DateTime::parse_from_rfc3339(item).ok())
            .is_some()
}

fn normalize_packet_evidence(
    mut data: serde_json::Value,
    submitted: Option<&TestEvidenceInput>,
    actor: &str,
    existing: Option<&serde_json::Value>,
) -> Result<serde_json::Value, AppError> {
    let object = data
        .as_object_mut()
        .ok_or_else(|| AppError::bad("Review evidence must be a packet object."))?;
    let checks = object
        .get_mut("checks")
        .and_then(|value| value.as_array_mut())
        .ok_or_else(|| AppError::bad("Review evidence must include the packet checks."))?;
    if checks.is_empty() {
        return Err(AppError::bad(
            "Review evidence must include at least one check.",
        ));
    }
    checks.retain(|check| {
        check.get("label").and_then(|label| label.as_str()) != Some("Test evidence")
    });
    let stored = existing
        .and_then(|value| value.get("test_evidence"))
        .filter(|value| valid_recorded_evidence(value))
        .cloned();
    let evidence = if let Some(submitted) = submitted {
        if submitted.command.trim().len() < 3 || submitted.result.trim().len() < 2 {
            return Err(AppError::bad(
                "Test evidence needs a command and a result before approval.",
            ));
        }
        Some(serde_json::json!({
            "command": submitted.command.trim(),
            "result": submitted.result.trim(),
            "recorded_by": actor,
            "recorded_at": Utc::now().to_rfc3339(),
        }))
    } else {
        stored
    };
    if let Some(evidence) = evidence {
        let command = evidence["command"].as_str().unwrap_or_default();
        let result = evidence["result"].as_str().unwrap_or_default();
        checks.push(serde_json::json!({
            "label":"Test evidence",
            "detail":format!("{command} — {result}"),
            "state":"done"
        }));
        object.insert("test_evidence".into(), evidence);
    } else {
        checks.push(serde_json::json!({
            "label":"Test evidence",
            "detail":"Attach the test command and result before owner approval.",
            "state":"missing"
        }));
        object.remove("test_evidence");
    }
    Ok(data)
}

fn evidence_is_complete(data: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(data)
        .ok()
        .and_then(|value| {
            let checks = value
                .get("checks")
                .and_then(|checks| checks.as_array())
                .cloned()?;
            valid_recorded_evidence(value.get("test_evidence")?).then_some(checks)
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
    AppJson(input): AppJson<EvidenceUpdate>,
) -> Result<Json<Packet>, AppError> {
    let who = session(&state, &headers).await?;
    let packet = sqlx::query_as::<_, Packet>("SELECT id,title,owner,status,data,created_at,approved_by,approved_at,source_url FROM packets WHERE id=? AND team_id=?").bind(&id).bind(&who.team_id).fetch_optional(&state.db).await?.ok_or_else(|| AppError::not_found("That review packet was not found in this team."))?;
    if packet.status == "approved" {
        return Err(AppError::conflict(
            "Approved packets are immutable. Create a new review packet for a changed decision.",
        ));
    }
    let existing: serde_json::Value = serde_json::from_str(&packet.data)
        .map_err(|_| AppError::service_unavailable("Stored review evidence is invalid."))?;
    let data = normalize_packet_evidence(
        input.data,
        input.test_evidence.as_ref(),
        &who.login,
        Some(&existing),
    )?
    .to_string();
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
    AppJson(input): AppJson<Approval>,
) -> Result<Json<Packet>, AppError> {
    let who = session(&state, &headers).await?;
    let packet = sqlx::query_as::<_, Packet>("SELECT id,title,owner,status,data,created_at,approved_by,approved_at,source_url FROM packets WHERE id=? AND team_id=?").bind(&id).bind(&who.team_id).fetch_optional(&state.db).await?.ok_or_else(|| AppError::not_found("That review packet was not found in this team."))?;
    if packet.owner != who.login {
        return Err(AppError::forbidden(
            "Only the required owner can approve this review packet.",
        ));
    }
    if packet.status == "approved" {
        return Err(AppError::conflict(
            "This packet is already approved and its approval is immutable.",
        ));
    }
    if !github_revision_is_current(&state, &who, &packet).await? {
        let refreshed = refresh_github_packet(&state, &who, &packet).await?;
        let revision: serde_json::Value = serde_json::from_str(&refreshed.data).unwrap_or_default();
        let head = revision
            .pointer("/github_revision/head_sha")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("a new revision");
        return Err(AppError::conflict(&format!(
            "This pull request changed to {head}. It was refreshed and needs new evidence before approval."
        )));
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
fn github_app_jwt(app_id: &str, private_key: &str) -> Result<String, AppError> {
    let now = Utc::now().timestamp() as usize;
    encode(
        &Header::new(Algorithm::RS256),
        &GithubJwtClaims {
            iss: app_id.to_string(),
            iat: now.saturating_sub(30),
            exp: now + 540,
        },
        &EncodingKey::from_rsa_pem(private_key.as_bytes())
            .map_err(|_| AppError::service_unavailable("The GitHub App key is invalid."))?,
    )
    .map_err(|_| AppError::service_unavailable("Could not sign the GitHub App request."))
}

fn html_attribute(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

async fn begin_github_app_manifest(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let who = session(&state, &headers).await?;
    let manifest_state = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO github_app_states (state,team_id,created_at) VALUES (?,?,?)")
        .bind(&manifest_state)
        .bind(&who.team_id)
        .bind(Utc::now().to_rfc3339())
        .execute(&state.db)
        .await?;
    let callback = format!(
        "{}/auth/github/created",
        state.identity.public_base.trim_end_matches('/')
    );
    let setup = format!(
        "{}/auth/github/installed",
        state.identity.public_base.trim_end_matches('/')
    );
    let suffix = manifest_state.chars().take(8).collect::<String>();
    let manifest = serde_json::json!({
        "name": format!("Diff Gate {}", suffix),
        "url": state.identity.public_base,
        "redirect_url": callback,
        "setup_url": setup,
        "public": false,
        "default_permissions": {
            "contents": "read",
            "pull_requests": "read",
            "metadata": "read"
        },
        "description": format!("Private pull-request review for {}", who.team_name)
    })
    .to_string();
    let body = format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>Connect GitHub App — Diff Gate</title></head><body><main><h1>Create your team GitHub App</h1><p>GitHub will create a private App with read-only access to pull requests and repository contents.</p><form method=\"post\" action=\"https://github.com/settings/apps/new\"><input type=\"hidden\" name=\"manifest\" value=\"{}\"><input type=\"hidden\" name=\"state\" value=\"{}\"><button type=\"submit\">Create GitHub App on GitHub</button></form><p><a href=\"/\">Cancel and return to Diff Gate</a></p></main></body></html>",
        html_attribute(&manifest),
        html_attribute(&manifest_state)
    );
    Ok(([(header::CONTENT_TYPE, "text/html; charset=utf-8")], body).into_response())
}

#[derive(Deserialize)]
struct GithubManifestCallback {
    code: String,
    state: String,
}
#[derive(Deserialize)]
struct GithubManifestApp {
    id: u64,
    pem: String,
    slug: String,
}
async fn github_app_created(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(query): axum::extract::Query<GithubManifestCallback>,
) -> Result<Redirect, AppError> {
    let who = session(&state, &headers).await?;
    let bound_team: Option<String> =
        sqlx::query_scalar("SELECT team_id FROM github_app_states WHERE state=?")
            .bind(&query.state)
            .fetch_optional(&state.db)
            .await?;
    if bound_team.as_deref() != Some(&who.team_id) {
        return Err(AppError::bad(
            "That GitHub App setup link expired. Start again.",
        ));
    }
    sqlx::query("DELETE FROM github_app_states WHERE state=?")
        .bind(&query.state)
        .execute(&state.db)
        .await?;
    let created = state
        .http
        .post(format!(
            "https://api.github.com/app-manifests/{}/conversions",
            query.code
        ))
        .header("user-agent", "diff-gate")
        .header("accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|_| AppError::service_unavailable("GitHub App setup could not be completed."))?
        .error_for_status()
        .map_err(|_| AppError::service_unavailable("GitHub rejected that App setup code."))?
        .json::<GithubManifestApp>()
        .await
        .map_err(|_| AppError::service_unavailable("GitHub returned an invalid App setup."))?;
    sqlx::query("INSERT INTO github_team_apps (team_id,app_id,private_key,app_slug,installation_id,updated_at) VALUES (?,?,?,?,NULL,?) ON CONFLICT(team_id) DO UPDATE SET app_id=excluded.app_id,private_key=excluded.private_key,app_slug=excluded.app_slug,installation_id=NULL,updated_at=excluded.updated_at")
        .bind(&who.team_id)
        .bind(created.id.to_string())
        .bind(created.pem)
        .bind(&created.slug)
        .bind(Utc::now().to_rfc3339())
        .execute(&state.db)
        .await?;
    Ok(Redirect::to(&github_install_url(&created.slug)))
}

#[derive(Deserialize)]
struct GithubInstallCallback {
    installation_id: String,
}
async fn github_app_installed(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(query): axum::extract::Query<GithubInstallCallback>,
) -> Result<Redirect, AppError> {
    let who = session(&state, &headers).await?;
    let team_app = team_github_app(&state.db, &who.team_id)
        .await?
        .ok_or_else(|| AppError::bad("Create the team GitHub App before installing it."))?;
    let jwt = github_app_jwt(&team_app.app_id, &team_app.private_key)?;
    let installed = state
        .http
        .get(format!(
            "https://api.github.com/app/installations/{}",
            query.installation_id
        ))
        .header("user-agent", "diff-gate")
        .header("accept", "application/vnd.github+json")
        .bearer_auth(jwt)
        .send()
        .await
        .map_err(|_| AppError::service_unavailable("GitHub installation could not be verified."))?
        .error_for_status()
        .map_err(|_| {
            AppError::forbidden("That installation does not belong to this team's GitHub App.")
        })?
        .json::<serde_json::Value>()
        .await
        .map_err(|_| AppError::service_unavailable("GitHub returned an invalid installation."))?;
    if installed
        .get("app_id")
        .and_then(|value| value.as_u64())
        .map(|value| value.to_string())
        != Some(team_app.app_id)
    {
        return Err(AppError::forbidden(
            "That installation does not belong to this team's GitHub App.",
        ));
    }
    sqlx::query("UPDATE github_team_apps SET installation_id=?,updated_at=? WHERE team_id=?")
        .bind(&query.installation_id)
        .bind(Utc::now().to_rfc3339())
        .bind(&who.team_id)
        .execute(&state.db)
        .await?;
    Ok(Redirect::to("/"))
}

async fn installation_token(state: &AppState, team_id: &str) -> Result<String, AppError> {
    let team_app = team_github_app(&state.db, team_id).await?;
    let (app_id, private_key, installation_id) = if let Some(team_app) = team_app {
        let installation_id = team_app.installation_id.ok_or_else(|| AppError::forbidden("Install the team GitHub App on the private repository before importing a pull request."))?;
        (team_app.app_id, team_app.private_key, installation_id)
    } else {
        if !state.github.app_ready() {
            return Err(AppError::service_unavailable(
                "Connect the Diff Gate GitHub App before importing a pull request.",
            ));
        }
        let installation_id = state.github.installation_for(team_id).ok_or_else(|| AppError::forbidden("No GitHub App installation is bound to this Sociobot team. Ask a team administrator to install and bind it."))?;
        (
            state.github.app_id.clone().unwrap_or_default(),
            state.github.private_key.clone().unwrap_or_default(),
            installation_id,
        )
    };
    let jwt = github_app_jwt(&app_id, &private_key)?;
    let endpoint = format!(
        "{}/app/installations/{}/access_tokens",
        state.github_api_base, installation_id
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
async fn github_packet_contents(
    state: &AppState,
    who: &Session,
    pr_url: &str,
) -> Result<(String, String, String, serde_json::Value), AppError> {
    let (owner, repo, number) = parse_pr_url(pr_url)?;
    let repository = format!(
        "{}/{}",
        owner.to_ascii_lowercase(),
        repo.to_ascii_lowercase()
    );
    let policy = repository_policy(&state.db, &who.team_id, &repository)
        .await?
        .ok_or_else(|| {
            AppError::bad(
                "Add a repository policy with sensitive paths and required owners before importing this pull request.",
            )
        })?;
    let token = installation_token(state, &who.team_id).await?;
    let base = format!(
        "{}/repos/{owner}/{repo}/pulls/{number}",
        state.github_api_base
    );
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
    let changed = github_changed_files(state, &base, &token).await?;
    let matches = matched_policy_rules(&changed, &policy.rules);
    let required_owners: BTreeSet<_> = matches
        .iter()
        .map(|(_, rule, _)| rule.required_owner.clone())
        .collect();
    if required_owners.len() > 1 {
        return Err(AppError::bad(
            "This pull request matches rules with different required owners. Split the change or align the repository policy.",
        ));
    }
    let packet_owner = required_owners
        .into_iter()
        .next()
        .unwrap_or_else(|| who.login.clone());
    let mut checks = vec![
        serde_json::json!({"label":"Pull request imported","detail":format!("PR #{number} by {}. {} changed files.", pull.user.login, changed.len()),"state":"ready"}),
        serde_json::json!({"label":"Repository policy","detail":format!("{} policy evaluated for this pull request.", policy.repository),"state":"ready"}),
    ];
    for (rule_index, rule, count) in matches {
        checks.push(serde_json::json!({
            "label":format!("Sensitive path: {}", rule.path),
            "detail":format!("{count} changed file(s) match this rule. Required owner: {}.", rule.required_owner),
            "state":"risk",
            "rule":rule_index,
        }));
    }
    let data = normalize_packet_evidence(
        serde_json::json!({"source":format!("PR #{number} · GitHub App import"),"changed":changed,"checks":checks,"repository_policy":policy.repository,"github_revision":{"head_sha":pull.head.sha,"state":"current"}}),
        None,
        &who.login,
        None,
    )?;
    Ok((pull.title, packet_owner, pull.html_url, data))
}

async fn import_github_pr(
    State(state): State<AppState>,
    headers: HeaderMap,
    AppJson(input): AppJson<ImportRequest>,
) -> Result<(StatusCode, Json<Packet>), AppError> {
    let who = session(&state, &headers).await?;
    let (title, owner, source_url, data) =
        github_packet_contents(&state, &who, &input.pr_url).await?;
    let packet = Packet {
        id: Uuid::new_v4().to_string(),
        title,
        owner,
        status: "needs review".into(),
        data: data.to_string(),
        created_at: Utc::now().to_rfc3339(),
        approved_by: None,
        approved_at: None,
        source_url: Some(source_url),
    };
    sqlx::query("INSERT INTO packets (id,team_id,title,owner,status,data,created_at,approved_by,approved_at,source_url) VALUES (?,?,?,?,?,?,?,?,?,?)").bind(&packet.id).bind(&who.team_id).bind(&packet.title).bind(&packet.owner).bind(&packet.status).bind(&packet.data).bind(&packet.created_at).bind(&packet.approved_by).bind(&packet.approved_at).bind(&packet.source_url).execute(&state.db).await?;
    audit(
        &state.db,
        &packet.id,
        &who.login,
        "imported",
        "GitHub App imported this pull request and evaluated its repository policy.",
    )
    .await?;
    Ok((StatusCode::CREATED, Json(packet)))
}

async fn refresh_github_packet(
    state: &AppState,
    who: &Session,
    packet: &Packet,
) -> Result<Packet, AppError> {
    let source_url = packet
        .source_url
        .as_deref()
        .ok_or_else(|| AppError::bad("Only GitHub-imported packets can be refreshed."))?;
    let (title, owner, source_url, data) = github_packet_contents(state, who, source_url).await?;
    sqlx::query("UPDATE packets SET title=?,owner=?,status='needs review',data=?,approved_by=NULL,approved_at=NULL,source_url=? WHERE id=? AND team_id=?")
        .bind(&title)
        .bind(&owner)
        .bind(data.to_string())
        .bind(&source_url)
        .bind(&packet.id)
        .bind(&who.team_id)
        .execute(&state.db)
        .await?;
    audit(
        &state.db,
        &packet.id,
        &who.login,
        "github_refreshed",
        "GitHub revision changed or was refreshed. Prior evidence and approval were cleared.",
    )
    .await?;
    sqlx::query_as("SELECT id,title,owner,status,data,created_at,approved_by,approved_at,source_url FROM packets WHERE id=? AND team_id=?")
        .bind(&packet.id)
        .bind(&who.team_id)
        .fetch_one(&state.db)
        .await
        .map_err(Into::into)
}

async fn refresh_packet(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Packet>, AppError> {
    let who = session(&state, &headers).await?;
    let packet = sqlx::query_as::<_, Packet>("SELECT id,title,owner,status,data,created_at,approved_by,approved_at,source_url FROM packets WHERE id=? AND team_id=?")
        .bind(&id).bind(&who.team_id).fetch_optional(&state.db).await?
        .ok_or_else(|| AppError::not_found("That review packet was not found in this team."))?;
    if packet.status == "approved" {
        return Err(AppError::conflict("Approved packets are immutable."));
    }
    if github_revision_is_current(&state, &who, &packet).await? {
        return Ok(Json(packet));
    }
    Ok(Json(refresh_github_packet(&state, &who, &packet).await?))
}

async fn github_revision_is_current(
    state: &AppState,
    who: &Session,
    packet: &Packet,
) -> Result<bool, AppError> {
    let Some(source_url) = packet.source_url.as_deref() else {
        return Ok(true);
    };
    let data: serde_json::Value = serde_json::from_str(&packet.data)
        .map_err(|_| AppError::service_unavailable("Stored review evidence is invalid."))?;
    let Some(expected) = data
        .pointer("/github_revision/head_sha")
        .and_then(serde_json::Value::as_str)
    else {
        return Ok(false);
    };
    let (owner, repo, number) = parse_pr_url(source_url)?;
    let token = installation_token(state, &who.team_id).await?;
    let base = format!(
        "{}/repos/{owner}/{repo}/pulls/{number}",
        state.github_api_base
    );
    let pull = state
        .http
        .get(base)
        .header("user-agent", "diff-gate")
        .header("accept", "application/vnd.github+json")
        .bearer_auth(token)
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
    Ok(pull.head.sha == expected)
}

fn path_matches_policy(path: &str, pattern: &str) -> bool {
    let pattern = pattern.trim_matches('/');
    if let Some(prefix) = pattern.strip_suffix("/**") {
        path.starts_with(&format!("{prefix}/"))
    } else if let Some(prefix) = pattern.strip_suffix('*') {
        path.starts_with(prefix)
    } else {
        path == pattern
    }
}

fn matched_policy_rules<'a>(
    changed: &[String],
    rules: &'a [PolicyRule],
) -> Vec<(usize, &'a PolicyRule, usize)> {
    rules
        .iter()
        .enumerate()
        .filter_map(|(index, rule)| {
            let count = changed
                .iter()
                .filter(|path| path_matches_policy(path, &rule.path))
                .count();
            (count > 0).then_some((index, rule, count))
        })
        .collect()
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
async fn stateful_production_guard(
    State(state): State<AppState>,
    request: axum::extract::Request,
    next: middleware::Next,
) -> Response {
    let path = request.uri().path();
    let anonymous_readiness_probe =
        request.method() == axum::http::Method::GET && path == "/api/auth/status";
    if is_production_host(request.headers())
        && !state.stateful_production_ready
        && !anonymous_readiness_probe
        && (path.starts_with("/api/") || path.starts_with("/auth/"))
    {
        return AppError::service_unavailable(
            "Diff Gate is waiting for its durable production storage configuration. Try again shortly.",
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
    fn invalid_json_input(rejection: &str) -> Self {
        let message = [
            ("title", "Invalid title. Add a text title and try again."),
            (
                "retention_days",
                "Invalid retention days. Use a whole number and try again.",
            ),
            (
                "repository",
                "Invalid repository. Use owner/repository and try again.",
            ),
            (
                "rules",
                "Invalid policy rules. Add each path and required owner.",
            ),
            (
                "data",
                "Invalid review data. Send the review data as a JSON object.",
            ),
            (
                "test_evidence",
                "Invalid test evidence. Add a command and result.",
            ),
            (
                "command",
                "Invalid test command. Add the command that was run.",
            ),
            ("result", "Invalid test result. Add the command result."),
            ("note", "Invalid approval note. Send text or omit the note."),
            (
                "pr_url",
                "Invalid pull request URL. Add a GitHub pull request URL.",
            ),
        ]
        .iter()
        .find_map(|(field, message)| {
            rejection
                .contains(&format!("`{field}`"))
                .then_some(*message)
        })
        .unwrap_or("Invalid request data. Send a complete JSON object and try again.");
        Self::bad(message)
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
async fn not_found_page(headers: HeaderMap) -> Response {
    let browser_navigation = headers
        .get(header::ACCEPT)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.contains("text/html"));
    // Chromium reports an error-level console message for a top-level document
    // with a 404 status, even when that document is a fully usable recovery
    // page. Navigation requests get the recovery page normally; API, monitor,
    // and command-line requests retain an HTTP 404. Both forms carry the same
    // explicit noindex route contract.
    let status = if browser_navigation {
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    };
    match tokio::fs::read("dist/index.html").await {
        Ok(body) => (
            status,
            [
                (header::CONTENT_TYPE, "text/html; charset=utf-8"),
                (
                    header::HeaderName::from_static("x-diff-gate-route"),
                    "not-found",
                ),
                (header::HeaderName::from_static("x-robots-tag"), "noindex"),
            ],
            body,
        )
            .into_response(),
        Err(_) => (
            status,
            [
                (header::CONTENT_TYPE, "text/plain; charset=utf-8"),
                (
                    header::HeaderName::from_static("x-diff-gate-route"),
                    "not-found",
                ),
                (header::HeaderName::from_static("x-robots-tag"), "noindex"),
            ],
            "Not found",
        )
            .into_response(),
    }
}
fn static_routes() -> Router<AppState> {
    let index = || get_service(ServeFile::new("dist/index.html"));
    Router::new()
        .route("/", index())
        .route("/demo", index())
        .route("/privacy", index())
        .route("/terms", index())
        .route("/404", get(not_found_page))
        .route_service("/404.css", ServeFile::new("dist/404.css"))
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
        .route("/api/packets/:id/refresh", post(refresh_packet))
        .route("/api/packets/:id/audit", get(list_packet_audit))
        .route("/api/settings", get(get_settings).put(update_settings))
        .route(
            "/api/repository-policies",
            get(list_repository_policies).put(save_repository_policy),
        )
        .route("/api/github/import", post(import_github_pr))
        .route("/api/auth/status", get(auth_status))
        .route("/auth/entra", get(entra_login))
        .route("/auth/callback", get(entra_callback))
        .route("/auth/github/new", get(begin_github_app_manifest))
        .route("/auth/github/created", get(github_app_created))
        .route("/auth/github/installed", get(github_app_installed))
        .route("/api/auth/signout", post(sign_out))
        .merge(static_routes())
        .layer(middleware::from_fn(cache_headers))
        .layer(middleware::from_fn_with_state(state.clone(), rate_limit));
    Router::new()
        .route("/health", get(health))
        .merge(protected)
        .layer(middleware::from_fn_with_state(
            state.clone(),
            stateful_production_guard,
        ))
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
    headers.insert(HeaderName::from_static("content-security-policy"),HeaderValue::from_static("default-src 'self'; img-src 'self' data:; style-src 'self'; script-src 'self'; connect-src 'self' https://api.sociobot.in https://github.com https://api.github.com; form-action 'self' https://github.com; frame-ancestors 'none'"));
    response
}
async fn create_schema(db: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS runtime_metadata (key TEXT PRIMARY KEY,value TEXT NOT NULL)",
    )
    .execute(db)
    .await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS teams (id TEXT PRIMARY KEY,name TEXT NOT NULL,created_at TEXT NOT NULL)").execute(db).await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS sessions (token TEXT PRIMARY KEY,team_id TEXT NOT NULL,login TEXT NOT NULL,expires_at TEXT NOT NULL)").execute(db).await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS oauth_states (state TEXT PRIMARY KEY,created_at TEXT NOT NULL)",
    )
    .execute(db)
    .await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS oauth_pkce (state TEXT PRIMARY KEY,nonce TEXT NOT NULL,verifier TEXT NOT NULL,created_at TEXT NOT NULL)").execute(db).await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS github_app_states (state TEXT PRIMARY KEY,team_id TEXT NOT NULL,created_at TEXT NOT NULL)").execute(db).await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS github_team_apps (team_id TEXT PRIMARY KEY,app_id TEXT NOT NULL,private_key TEXT NOT NULL,app_slug TEXT NOT NULL,installation_id TEXT,updated_at TEXT NOT NULL)").execute(db).await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS packets (id TEXT PRIMARY KEY,team_id TEXT NOT NULL DEFAULT '',title TEXT NOT NULL,owner TEXT NOT NULL,status TEXT NOT NULL,data TEXT NOT NULL,created_at TEXT NOT NULL,approved_by TEXT,approved_at TEXT,source_url TEXT)").execute(db).await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS packet_audit (id TEXT PRIMARY KEY,packet_id TEXT NOT NULL,actor TEXT NOT NULL,action TEXT NOT NULL,detail TEXT NOT NULL,created_at TEXT NOT NULL)").execute(db).await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS team_settings (team_id TEXT PRIMARY KEY,retention_days INTEGER NOT NULL CHECK(retention_days BETWEEN 1 AND 3650))").execute(db).await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS repository_policies (team_id TEXT NOT NULL,repository TEXT NOT NULL,rules TEXT NOT NULL,updated_at TEXT NOT NULL,PRIMARY KEY(team_id,repository))").execute(db).await?;
    Ok(())
}
async fn durable_storage_id(db: &SqlitePool) -> Result<String, sqlx::Error> {
    sqlx::query("INSERT OR IGNORE INTO runtime_metadata (key,value) VALUES ('storage_id',?)")
        .bind(Uuid::new_v4().to_string())
        .execute(db)
        .await?;
    sqlx::query_scalar("SELECT value FROM runtime_metadata WHERE key='storage_id'")
        .fetch_one(db)
        .await
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
        Err(_) => (DURABLE_DATABASE_URL.into(), "generated default"),
    };
    if !std::path::Path::new("/data").exists() {
        std::fs::create_dir_all("/data").ok();
    }
    let identity = EntraConfig::from_env();
    let github = GithubConfig::from_env();
    let stateful_production_ready = supplied_stateful_production_contract();
    info!(
        database_config,
        stateful_production_ready,
        entra_identity = identity.ready(),
        github_app = github.app_ready(),
        "Diff Gate starting; database uses the supplied or generated configuration"
    );
    let db = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;
    create_schema(&db).await?;
    let storage_id = durable_storage_id(&db).await?;
    purge_all_expired_data(&db).await?;
    let state = AppState {
        db,
        build,
        storage_id,
        stateful_production_ready,
        limits: Arc::new(Mutex::new(HashMap::new())),
        identity,
        github,
        github_api_base: "https://api.github.com".into(),
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
            storage_id: durable_storage_id(&db).await.unwrap(),
            stateful_production_ready: false,
            limits: Arc::new(Mutex::new(HashMap::new())),
            identity: EntraConfig::default(),
            github: GithubConfig::default(),
            github_api_base: "https://api.github.com".into(),
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
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&body).unwrap()["build"],
            "test-sha"
        );
    }
    #[tokio::test]
    async fn regression_verifier_17_port_only_public_runtime_fails_closed_without_breaking_cold_landing(
    ) {
        // Verification 17 found the candidate running publicly with only PORT,
        // which meant multiple replicas wrote independent ephemeral SQLite
        // stores. Local PORT-only startup remains supported, but that same
        // incomplete configuration must never serve production state traffic.
        let (app, _) = test_app().await;
        let health = app
            .clone()
            .oneshot(
                Request::get("/health")
                    .header(header::HOST, "agent-diff-gate.sociobot.in")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(health.status(), StatusCode::SERVICE_UNAVAILABLE);
        let health_body = to_bytes(health.into_body(), usize::MAX).await.unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&health_body).unwrap()["status"],
            "unsafe_configuration"
        );

        let api = app
            .clone()
            .oneshot(
                Request::get("/api/auth/status")
                    .header(header::HOST, "agent-diff-gate.sociobot.in")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(api.status(), StatusCode::OK);
        let api_body = to_bytes(api.into_body(), usize::MAX).await.unwrap();
        let readiness = serde_json::from_slice::<serde_json::Value>(&api_body).unwrap();
        assert_eq!(readiness["service_ready"], false);
        assert_eq!(readiness["authenticated"], false);

        let packets = app
            .oneshot(
                Request::get("/api/packets")
                    .header(header::HOST, "agent-diff-gate.sociobot.in")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(packets.status(), StatusCode::SERVICE_UNAVAILABLE);
    }
    #[tokio::test]
    async fn missing_routes_keep_a_http_404_for_non_navigation_requests() {
        let (app, _) = test_app().await;
        for path in ["/this-route-does-not-exist", "/404"] {
            let response = app
                .clone()
                .oneshot(Request::get(path).body(axum::body::Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::NOT_FOUND, "{path}");
            assert_eq!(response.headers()["x-diff-gate-route"], "not-found");
            assert_eq!(response.headers()["x-robots-tag"], "noindex");
            assert_eq!(
                response.headers()[header::CONTENT_TYPE],
                "text/html; charset=utf-8"
            );
            let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
            assert!(String::from_utf8_lossy(&body).contains("id=\"app\""));
        }
    }
    #[tokio::test]
    async fn missing_navigation_returns_the_recovery_view_without_a_document_error() {
        let (app, _) = test_app().await;
        let response = app
            .oneshot(
                Request::get("/this-route-does-not-exist")
                    .header(header::ACCEPT, "text/html,application/xhtml+xml")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()["x-diff-gate-route"], "not-found");
        assert_eq!(response.headers()["x-robots-tag"], "noindex");
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
    async fn malformed_json_returns_a_stable_actionable_json_error() {
        let (app, _) = test_app().await;
        let response = app
            .oneshot(
                Request::post("/api/packets")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert!(response.headers()[header::CONTENT_TYPE]
            .to_str()
            .unwrap()
            .starts_with("application/json"));
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&body).unwrap(),
            serde_json::json!({"error":"Invalid title. Add a text title and try again."})
        );
        assert!(!String::from_utf8_lossy(&body).contains("deserialize"));
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
        sqlx::query("INSERT INTO packets (id,team_id,title,owner,status,data,created_at) VALUES ('packet-b','b','Private change','bea','needs review',?,?)")
            .bind(serde_json::json!({"checks":[{"label":"Review","state":"done"}],"test_evidence":{"command":"cargo test","result":"passed","recorded_by":"bea","recorded_at":now}}).to_string())
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
        let forged_evidence = app
            .clone()
            .oneshot(
                Request::put("/api/packets/packet")
                    .header("cookie", "diff_gate_session=owner-token")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from("{\"data\":{\"checks\":[{\"label\":\"Test evidence\",\"detail\":\"Attach the test command and result before owner approval.\",\"state\":\"done\"}]}}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(forged_evidence.status(), StatusCode::OK);
        let forged_approval = app
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
        assert_eq!(forged_approval.status(), StatusCode::BAD_REQUEST);
        let evidence = app
            .clone()
            .oneshot(
                Request::put("/api/packets/packet")
                    .header("cookie", "diff_gate_session=owner-token")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from("{\"data\":{\"checks\":[{\"label\":\"Review complete\",\"state\":\"done\"}]},\"test_evidence\":{\"command\":\"cargo test\",\"result\":\"24 passed\"}}"))
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
        assert!(state.contains("cargo test"));
        assert!(state.contains("\"recorded_by\":\"owner"));
        let audit_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM packet_audit WHERE packet_id='packet' AND action='evidence_updated'").fetch_one(&db).await.unwrap();
        assert_eq!(audit_count, 2);
    }
    #[test]
    fn entra_and_github_installations_are_configured_per_team() {
        let identity = EntraConfig {
            authority: Some(SOCIOBOT_AUTHORITY.into()),
            client_id: Some("client-id".into()),
            tenant_id: SOCIOBOT_TENANT_ID.into(),
            public_base: "https://agent-diff-gate.sociobot.in".into(),
            team_claim: "oid".into(),
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
            "https://agent-diff-gate.sociobot.in/auth/callback"
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
    #[test]
    fn missing_public_base_never_redirects_entra_to_localhost() {
        let identity = EntraConfig {
            authority: Some(SOCIOBOT_AUTHORITY.into()),
            client_id: Some(SOCIOBOT_CLIENT_ID.into()),
            tenant_id: SOCIOBOT_TENANT_ID.into(),
            public_base: configured_public_base(None),
            team_claim: "oid".into(),
        };
        assert_eq!(
            identity.callback_url(),
            "https://agent-diff-gate.sociobot.in/auth/callback"
        );
        assert_eq!(
            configured_public_base(Some("   ")),
            "https://agent-diff-gate.sociobot.in"
        );
    }
    #[tokio::test]
    async fn live_identity_defaults_to_sociobot_and_uses_pkce_without_a_client_secret() {
        let db = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        create_schema(&db).await.unwrap();
        let identity = EntraConfig {
            authority: Some(SOCIOBOT_AUTHORITY.into()),
            client_id: Some(SOCIOBOT_CLIENT_ID.into()),
            tenant_id: SOCIOBOT_TENANT_ID.into(),
            public_base: "https://agent-diff-gate.sociobot.in".into(),
            team_claim: "oid".into(),
        };
        let service = app(AppState {
            db,
            build: "identity-regression".into(),
            storage_id: "identity-fixture".into(),
            stateful_production_ready: true,
            limits: Arc::new(Mutex::new(HashMap::new())),
            identity,
            github: GithubConfig::default(),
            github_api_base: "https://api.github.com".into(),
            http: Client::new(),
        });
        let status = service
            .clone()
            .oneshot(
                Request::get("/api/auth/status")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let status_body = to_bytes(status.into_body(), usize::MAX).await.unwrap();
        let status_json: serde_json::Value = serde_json::from_slice(&status_body).unwrap();
        assert_eq!(status_json["entra_sign_in_configured"], true);
        assert_eq!(status_json["github_app_setup_available"], true);
        let redirect = service
            .oneshot(
                Request::get("/auth/entra")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(redirect.status(), StatusCode::TEMPORARY_REDIRECT);
        let location = Url::parse(redirect.headers()[header::LOCATION].to_str().unwrap()).unwrap();
        let query: HashMap<_, _> = location.query_pairs().into_owned().collect();
        assert_eq!(location.host_str(), Some("sociobotcustomers.ciamlogin.com"));
        assert_eq!(
            query.get("client_id").map(String::as_str),
            Some(SOCIOBOT_CLIENT_ID)
        );
        assert_eq!(
            query.get("redirect_uri").map(String::as_str),
            Some("https://agent-diff-gate.sociobot.in/auth/callback")
        );
        assert_eq!(
            query.get("code_challenge_method").map(String::as_str),
            Some("S256")
        );
        assert!(query
            .get("code_challenge")
            .is_some_and(|value| value.len() == 43));
        assert!(!query.contains_key("client_secret"));
    }
    #[tokio::test]
    async fn entra_callback_error_renders_recovery_before_requiring_code() {
        let db = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        create_schema(&db).await.unwrap();
        sqlx::query("INSERT INTO oauth_pkce (state,nonce,verifier,created_at) VALUES ('cancel-state','nonce','verifier',?)")
            .bind(Utc::now().to_rfc3339())
            .execute(&db)
            .await
            .unwrap();
        let identity = EntraConfig {
            authority: Some(SOCIOBOT_AUTHORITY.into()),
            client_id: Some(SOCIOBOT_CLIENT_ID.into()),
            tenant_id: SOCIOBOT_TENANT_ID.into(),
            public_base: "https://agent-diff-gate.sociobot.in".into(),
            team_claim: "oid".into(),
        };
        let service = app(AppState {
            db: db.clone(),
            build: "callback-regression".into(),
            storage_id: "callback-fixture".into(),
            stateful_production_ready: true,
            limits: Arc::new(Mutex::new(HashMap::new())),
            identity,
            github: GithubConfig::default(),
            github_api_base: "https://api.github.com".into(),
            http: Client::new(),
        });

        let response = service
            .oneshot(
                Request::get("/auth/callback?error=access_denied&error_description=User%20cancelled%20%3Cscript%3E&state=cancel-state")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers()[header::CONTENT_TYPE],
            "text/html; charset=utf-8"
        );
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-cache");
        assert_eq!(response.headers()["x-robots-tag"], "noindex");
        assert!(response.headers()["content-security-policy"]
            .to_str()
            .unwrap()
            .contains("frame-ancestors 'none'"));
        let body = String::from_utf8(
            to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        assert!(body.contains("<h1>Sign-in did not complete</h1>"));
        assert!(body.contains("Sign-in was cancelled or your account did not grant access."));
        assert!(body.contains("href=\"/auth/entra\">Try sign-in again</a>"));
        assert!(body.contains("href=\"/\">Return to Diff Gate</a>"));
        assert!(body.contains("href=\"/?demo=1\">Try it with sample data</a>"));
        assert_eq!(body.matches("<h1").count(), 1);
        assert!(!body.contains("missing field"));
        assert!(!body.contains("<script>"));
        let pending: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM oauth_pkce WHERE state='cancel-state'")
                .fetch_one(&db)
                .await
                .unwrap();
        assert_eq!(pending, 0);
    }
    #[tokio::test]
    async fn durable_storage_identity_survives_database_reopen() {
        let path = std::env::temp_dir().join(format!("diff-gate-storage-{}.db", Uuid::new_v4()));
        let url = format!("sqlite:{}?mode=rwc", path.display());
        let first = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(&url)
            .await
            .unwrap();
        create_schema(&first).await.unwrap();
        let first_id = durable_storage_id(&first).await.unwrap();
        first.close().await;
        let replacement = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(&url)
            .await
            .unwrap();
        create_schema(&replacement).await.unwrap();
        assert_eq!(durable_storage_id(&replacement).await.unwrap(), first_id);
        replacement.close().await;
        std::fs::remove_file(path).unwrap();
    }
    #[tokio::test]
    async fn github_app_manifest_is_read_only_and_bound_to_the_signed_in_team() {
        let (service, db) = test_app().await;
        let future = (Utc::now() + ChronoDuration::days(1)).to_rfc3339();
        sqlx::query("INSERT INTO teams (id,name,created_at) VALUES ('team-a','Alpha',?),('team-b','Beta',?)")
            .bind(&future)
            .bind(&future)
            .execute(&db)
            .await
            .unwrap();
        sqlx::query("INSERT INTO sessions (token,team_id,login,expires_at) VALUES ('a-token','team-a','owner-a',?),('b-token','team-b','owner-b',?)")
            .bind(&future)
            .bind(&future)
            .execute(&db)
            .await
            .unwrap();
        let response = service
            .clone()
            .oneshot(
                Request::get("/auth/github/new")
                    .header("cookie", "diff_gate_session=a-token")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = String::from_utf8(
            to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        assert!(body.contains("https://github.com/settings/apps/new"));
        assert!(body.contains("&quot;pull_requests&quot;:&quot;read&quot;"));
        assert!(body.contains("&quot;contents&quot;:&quot;read&quot;"));
        assert!(!body.contains("&quot;contents&quot;:&quot;write&quot;"));
        let (manifest_state, bound_team): (String, String) =
            sqlx::query_as("SELECT state,team_id FROM github_app_states")
                .fetch_one(&db)
                .await
                .unwrap();
        assert_eq!(bound_team, "team-a");
        let cross_team = service
            .oneshot(
                Request::get(format!(
                    "/auth/github/created?state={manifest_state}&code=attacker-code"
                ))
                .header("cookie", "diff_gate_session=b-token")
                .body(axum::body::Body::empty())
                .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(cross_team.status(), StatusCode::BAD_REQUEST);
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
                serde_json::json!({"filename":"schema/user.graphql"}),
                serde_json::json!({"filename":"infra/production.tf"}),
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
            storage_id: "github-fixture".into(),
            stateful_production_ready: false,
            limits: Arc::new(Mutex::new(HashMap::new())),
            identity: EntraConfig::default(),
            github: GithubConfig::default(),
            github_api_base: "https://api.github.com".into(),
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
        let rules = vec![
            PolicyRule {
                path: "schema/**".into(),
                required_owner: "database-owner".into(),
            },
            PolicyRule {
                path: "infra/**".into(),
                required_owner: "platform-owner".into(),
            },
        ];
        let matches = matched_policy_rules(&changed, &rules);
        assert_eq!(
            matches
                .iter()
                .map(|(_, rule, _)| rule.required_owner.as_str())
                .collect::<Vec<_>>(),
            vec!["database-owner", "platform-owner"]
        );
        server.abort();
    }
    #[tokio::test]
    async fn github_import_rejects_more_than_10000_files() {
        async fn files() -> Json<Vec<serde_json::Value>> {
            Json(
                (0..100)
                    .map(|index| serde_json::json!({"filename": format!("src/file-{index}.ts")}))
                    .collect(),
            )
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
            build: "limit-fixture".into(),
            storage_id: "limit-fixture".into(),
            stateful_production_ready: false,
            limits: Arc::new(Mutex::new(HashMap::new())),
            identity: EntraConfig::default(),
            github: GithubConfig::default(),
            github_api_base: "https://api.github.com".into(),
            http: Client::new(),
        };
        let error = github_changed_files(
            &state,
            &format!("http://{address}/pulls/42"),
            "fixture-installation-token",
        )
        .await
        .unwrap_err();
        assert_eq!(error.status, StatusCode::BAD_REQUEST);
        assert!(error.message.contains("more than 10,000"));
        server.abort();
    }
    #[tokio::test]
    async fn github_revision_change_refreshes_packet_and_blocks_approval() {
        const TEST_KEY: &str = "-----BEGIN PRIVATE KEY-----\nMIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQDt8tpZ0rgHTDK7\n1WACFsLwtciBoKnzGu6c6IxNA6UZE0VNFW3bhjzT059irao7XZYr7ZR2TqM9f7/X\nclp2T8fdgJmolJDFfcPp9/G901Oe1Pno3dZjooZG9OIRm00i5H1Zi37DAeffw3K3\n3T9WI6i4d7dnnrZvJ9lyRrUlbq4WyzT5dukTVcFeuDqjm4OmUjTfEITEztt8uuNu\npF7CehLQFpMF7Nt+HzEasjLW48u6lgOMnyJFtzCEJr4BSKcci9qh6vBZO4pXL2t6\nTinHuoi52t7Ih8jUfwEpWVTFCAAK9kq5+STWCvf6z8oJGNJ+z3zoLuPC2mTcr7UV\ncBXqdpvpAgMBAAECggEANXJNkkxu8pCueptQX9e9/LRQL7GnSsg7XXosfWX6sPmv\noMNV9C+gPRI1JESOzpvUTdSk+rfqGbe2nw17/UQpR/sJSKDqLbn0hfqfzXwItc3v\nvlsJu0J3t7tshfjkqBg7gaAAHowwiYXMoDjtb4s97AVT6E3xe2EviegQ6zIDn3Gh\ncVvM9OPubcjCcGG1P8ZLKI/rfDujdVO02napgSULHdvkE+XLe9iUgmjl+5zf1Zq6\n8Hl80OauumNs0ax319GvgrQAF7sze121yqKUmX5ed3mi38i5bB49t8n71LOc9TwP\nJD+GT11toQcb2f/Cwn7PRutvOIR+xeTtFbCYDVT1KQKBgQD89n/sBjD14V4Azz7Z\n35XgGtrnykP6x5TtJGL5SWU30hnrbklyvekeCpgluMw3kxXWM/3Aw0NdfV22EKzj\n+o2tavu47fo1JxSzodgEYYmi7tLtp5pRQDRrN7XVw/qHCGTrfgstmv3VRoAhFdND\no2PnaLmsYk5UtpEiNsMrbmoSRQKBgQDwzjSxpoyV90r4aXMzA/NCxzvqoCS2cuMF\nptxZIzs7cVbjMalQvZEVJCnetdHi6JR4qW3Nvhx+npPUR29u2jZTVAmguXKLdpF1\njgpI6OQ2xpGKM/xyE+16i1fNCq3qqTbNpq33hzopBJddLVQiudQxbX+PtJWATbS3\ngFhZcoKPVQKBgQDpxY+genRCtpwd2Wi3BjZGnerRLI44MrtBkE/bGuXseUC03v4H\niNPnjFjg+2/WqBoVE4Uc4BbgThwNRknQgdrueaDZXSvOdShffWDZY55DsbvCHxKw\npcoLj7d+LpfWtH43VwtTgRm1QGrmqHnN1zBbSd/VHCBRj0p+uOcSuv5RlQKBgQDq\nAvoiSgAFHLS2g4N36DbWhlcrw0TqKOuF6onn9dzx/0q4ruIjnJUJPoOR8o9tOyhN\nuhkC/+UhB2oRuPoJd/WjNN/GWXF/JlJlMwu7ntdog7+b1rlVAxidJhzFHcO1b4va\nfkhBbCCRC+0sl4hT1tLm1cpJFOzUKq+cRBWXlzhZoQKBgHyGEMhogoYlnQ5KNlg9\nXTJfuLRxnEq1zC1Se89yLwPh/4oFAluyK+eJHSeKFrGe9HOBZWf/yNcfK2FayJPY\nl+haL38Zn9tcrg25s8GwcUjgm7LFI3DHG2x3u68dP5BJZEDbBi3wnEfoQR4O3Iq0\nMlIKpV75uK9nawErnwQd4KFy\n-----END PRIVATE KEY-----";
        async fn token() -> Json<serde_json::Value> {
            Json(serde_json::json!({"token":"fixture"}))
        }
        async fn pull() -> Json<serde_json::Value> {
            Json(
                serde_json::json!({"title":"Changed after review","html_url":"https://github.com/acme/api/pull/42","user":{"login":"agent"},"head":{"sha":"after-sha"}}),
            )
        }
        async fn files() -> Json<Vec<serde_json::Value>> {
            Json(vec![serde_json::json!({"filename":"schema/user.graphql"})])
        }
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(
                listener,
                Router::new()
                    .route("/app/installations/1/access_tokens", post(token))
                    .route("/repos/acme/api/pulls/42", get(pull))
                    .route("/repos/acme/api/pulls/42/files", get(files)),
            )
            .await
            .unwrap();
        });
        let db = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        create_schema(&db).await.unwrap();
        let future = (Utc::now() + ChronoDuration::days(1)).to_rfc3339();
        sqlx::query("INSERT INTO teams (id,name,created_at) VALUES ('team','Team',?)")
            .bind(&future)
            .execute(&db)
            .await
            .unwrap();
        sqlx::query("INSERT INTO sessions (token,team_id,login,expires_at) VALUES ('token','team','owner',?)").bind(&future).execute(&db).await.unwrap();
        sqlx::query("INSERT INTO repository_policies (team_id,repository,rules,updated_at) VALUES ('team','acme/api',?,?)")
            .bind("[{\"path\":\"schema/**\",\"required_owner\":\"owner\"}]").bind(&future).execute(&db).await.unwrap();
        let ready = serde_json::json!({"changed":["schema/old.graphql"],"checks":[{"label":"Review","state":"done"}],"github_revision":{"head_sha":"before-sha","state":"current"},"test_evidence":{"command":"cargo test","result":"passed","recorded_by":"owner","recorded_at":future}}).to_string();
        sqlx::query("INSERT INTO packets (id,team_id,title,owner,status,data,created_at,source_url) VALUES ('packet','team','Before','owner','needs review',?,?,'https://github.com/acme/api/pull/42')").bind(ready).bind(&future).execute(&db).await.unwrap();
        let app = app(AppState {
            db: db.clone(),
            build: "test".into(),
            storage_id: "test".into(),
            stateful_production_ready: false,
            limits: Arc::new(Mutex::new(HashMap::new())),
            identity: EntraConfig::default(),
            github: GithubConfig {
                app_id: Some("1".into()),
                private_key: Some(TEST_KEY.into()),
                app_slug: Some("fixture".into()),
                installations: HashMap::from([("team".into(), "1".into())]),
            },
            github_api_base: format!("http://{address}"),
            http: Client::new(),
        });
        let response = app
            .oneshot(
                Request::post("/api/packets/packet/approve")
                    .header("cookie", "diff_gate_session=token")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CONFLICT);
        let stored: (String, String) =
            sqlx::query_as("SELECT status,data FROM packets WHERE id='packet'")
                .fetch_one(&db)
                .await
                .unwrap();
        let refreshed: serde_json::Value = serde_json::from_str(&stored.1).unwrap();
        assert_eq!(stored.0, "needs review");
        assert_eq!(refreshed["github_revision"]["head_sha"], "after-sha");
        assert!(refreshed.get("test_evidence").is_none());
        server.abort();
    }
    #[tokio::test]
    async fn repository_policy_is_team_scoped_and_requires_its_own_paths_and_owner() {
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
        sqlx::query("INSERT INTO sessions (token,team_id,login,expires_at) VALUES ('token','team','owner',?),('other-token','other','other',?)")
            .bind(&future).bind(&future).execute(&db).await.unwrap();
        let saved = app.clone().oneshot(Request::put("/api/repository-policies")
            .header("cookie", "diff_gate_session=token").header("content-type", "application/json")
            .body(axum::body::Body::from("{\"repository\":\"Acme/Service\",\"rules\":[{\"path\":\"schema/**\",\"required_owner\":\"database-owner\"}]}"))
            .unwrap()).await.unwrap();
        assert_eq!(saved.status(), StatusCode::OK);
        let own = app
            .clone()
            .oneshot(
                Request::get("/api/repository-policies")
                    .header("cookie", "diff_gate_session=token")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let own_body = to_bytes(own.into_body(), usize::MAX).await.unwrap();
        assert!(std::str::from_utf8(&own_body)
            .unwrap()
            .contains("database-owner"));
        let hidden = app
            .oneshot(
                Request::get("/api/repository-policies")
                    .header("cookie", "diff_gate_session=other-token")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let hidden_body = to_bytes(hidden.into_body(), usize::MAX).await.unwrap();
        assert_eq!(hidden_body.as_ref(), b"[]");
        assert!(path_matches_policy("schema/user.graphql", "schema/**"));
        assert!(!path_matches_policy("src/schema/user.graphql", "schema/**"));
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
    async fn retention_limits_default_and_read_cleanup_are_enforced() {
        let (service, db) = test_app().await;
        let future = (Utc::now() + ChronoDuration::days(1)).to_rfc3339();
        let expired = (Utc::now() - ChronoDuration::days(DEFAULT_RETENTION_DAYS + 1)).to_rfc3339();
        sqlx::query("INSERT INTO teams (id,name,created_at) VALUES ('team','Team',?)")
            .bind(&future)
            .execute(&db)
            .await
            .unwrap();
        sqlx::query("INSERT INTO sessions (token,team_id,login,expires_at) VALUES ('token','team','owner',?)")
            .bind(&future)
            .execute(&db)
            .await
            .unwrap();
        sqlx::query("INSERT INTO packets (id,team_id,title,owner,status,data,created_at) VALUES ('expired','team','Expired','owner','needs review','{}',?)")
            .bind(&expired)
            .execute(&db)
            .await
            .unwrap();
        let request = |body: &'static str| {
            Request::put("/api/settings")
                .header("cookie", "diff_gate_session=token")
                .header("content-type", "application/json")
                .body(axum::body::Body::from(body))
                .unwrap()
        };
        let defaults = service
            .clone()
            .oneshot(
                Request::get("/api/settings")
                    .header("cookie", "diff_gate_session=token")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let defaults: serde_json::Value =
            serde_json::from_slice(&to_bytes(defaults.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert_eq!(defaults["retention_days"], DEFAULT_RETENTION_DAYS);
        for body in ["{\"retention_days\":0}", "{\"retention_days\":3651}"] {
            assert_eq!(
                service
                    .clone()
                    .oneshot(request(body))
                    .await
                    .unwrap()
                    .status(),
                StatusCode::BAD_REQUEST
            );
        }
        let packets = service
            .oneshot(
                Request::get("/api/packets")
                    .header("cookie", "diff_gate_session=token")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(packets.status(), StatusCode::OK);
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM packets WHERE id='expired'")
            .fetch_one(&db)
            .await
            .unwrap();
        assert_eq!(count, 0);
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
        sqlx::query("INSERT INTO packets (id,team_id,title,owner,status,data,created_at) VALUES ('packet','team','Ready','owner','needs review',?,?)")
            .bind(serde_json::json!({"checks":[{"label":"Review","state":"done"}],"test_evidence":{"command":"cargo test","result":"passed","recorded_by":"owner","recorded_at":future}}).to_string())
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
