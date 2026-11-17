use std::collections::{HashMap, VecDeque};
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::extract::{ConnectInfo, FromRequestParts, Request, State};
use axum::http::request::Parts;
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode, header};
use axum::middleware::{Next, from_fn_with_state};
use axum::response::{IntoResponse, Response};
use axum::{Json, Router};
use fluent_bundle::FluentArgs;
use serde::Deserialize;
use sqlx::PgConnection;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::api::AppState;
use crate::configuration::{FirstOrganizer, Network, PublicUrl};
use crate::mail::{self, Mail};
use crate::members::{self, Member};
use crate::{localization, secret};

const SESSION_DAYS: i32 = 14;
const LINKS_PER_ADDRESS_AND_HOUR: i64 = 3;
const REQUESTS_PER_NETWORK_ADDRESS_AND_HOUR: usize = 20;
const HOUR: Duration = Duration::from_hours(1);
const SECURE_COOKIE_NAME: &str = "__Host-adunata_session";

/// Rejects every request that may change data and comes from a page on another origin. A request without an `Origin` header does not come from a browser page and passes.
pub fn with_origin_check(api: Router, public_url: &PublicUrl) -> Router {
    api.layer(from_fn_with_state(public_url.origin(), check_origin))
}

async fn check_origin(State(origin): State<String>, request: Request, next: Next) -> Response {
    let safe = matches!(*request.method(), Method::GET | Method::HEAD);
    let foreign = request
        .headers()
        .get(header::ORIGIN)
        .is_some_and(|sent| sent.as_bytes() != origin.as_bytes());
    if !safe && foreign {
        return StatusCode::FORBIDDEN.into_response();
    }
    next.run(request).await
}

/// The mail address for which someone asks for a sign-in link.
#[derive(Clone, Debug, Deserialize, ToSchema)]
pub struct SignInRequest {
    pub mail: String,
}

/// What Adunata did about the first organizer at start-up.
#[derive(Debug, PartialEq, Eq)]
pub enum FirstOrganizerOutcome {
    Created,
    AlreadyPresent,
    NobodyCanSignIn,
}

/// Creates the first organizer from the settings if the instance has no active organizer yet.
pub async fn ensure_first_organizer(
    pool: &sqlx::PgPool,
    first: Option<&FirstOrganizer>,
    language: &str,
) -> Result<FirstOrganizerOutcome, sqlx::Error> {
    let mut connection = pool.acquire().await?;
    if members::has_active_organizer(&mut connection).await? {
        return Ok(FirstOrganizerOutcome::AlreadyPresent);
    }
    let Some(first) = first else {
        return Ok(FirstOrganizerOutcome::NobodyCanSignIn);
    };
    members::grant_organizer(
        &mut connection,
        &first.mail,
        &first.first_name,
        &first.last_name,
        language,
    )
    .await?;
    Ok(FirstOrganizerOutcome::Created)
}

/// Deletes sessions that have ended and sign-in links that are used or expired.
pub async fn delete_expired(connection: &mut PgConnection) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM sessions WHERE expires_at <= now()")
        .execute(&mut *connection)
        .await?;
    sqlx::query("DELETE FROM sign_in_links WHERE used_at IS NOT NULL OR expires_at <= now()")
        .execute(connection)
        .await
        .map(|_| ())
}

/// Counts sign-in requests per network address in memory, which is enough for one process.
#[derive(Clone, Default)]
pub struct SignInLimiter {
    requests: Arc<Mutex<HashMap<IpAddr, VecDeque<Instant>>>>,
}

impl SignInLimiter {
    /// Records a request and tells whether the address stays within its limit for the past hour.
    fn allow(&self, address: IpAddr, now: Instant) -> bool {
        let mut requests = self
            .requests
            .lock()
            .expect("no thread panics while holding the limiter");
        requests.retain(|_, times| {
            while times
                .front()
                .is_some_and(|time| now.duration_since(*time) >= HOUR)
            {
                times.pop_front();
            }
            !times.is_empty()
        });
        let times = requests.entry(address).or_default();
        if times.len() >= REQUESTS_PER_NETWORK_ADDRESS_AND_HOUR {
            return false;
        }
        times.push_back(now);
        true
    }
}

/// The network address of the client. Behind a trusted proxy it is the rightmost entry of `X-Forwarded-For` that is no trusted proxy itself. Entries further left are whatever the client sent.
pub struct ClientAddress(pub Option<IpAddr>);

impl FromRequestParts<AppState> for ClientAddress {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let peer = parts
            .extensions
            .get::<ConnectInfo<SocketAddr>>()
            .map(|ConnectInfo(address)| address.ip());
        Ok(Self(peer.map(|peer| {
            client_address(peer, &parts.headers, &state.trusted_proxies)
        })))
    }
}

fn client_address(peer: IpAddr, headers: &HeaderMap, trusted: &[Network]) -> IpAddr {
    let is_trusted = |address: IpAddr| trusted.iter().any(|network| network.contains(address));
    if !is_trusted(peer) {
        return peer;
    }
    let forwarded: Vec<IpAddr> = headers
        .get_all("x-forwarded-for")
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(','))
        .map(|entry| entry.trim().parse::<IpAddr>().ok())
        .collect::<Option<_>>()
        .unwrap_or_default();
    forwarded
        .into_iter()
        .rev()
        .find(|address| !is_trusted(*address))
        .unwrap_or(peer)
}

/// The signed-in organizer. Organizer routes take it as an argument and so reject every request without a valid session with status 401. The session counts only while its member is active and holds the organizer role.
pub struct Organizer(pub Member);

impl FromRequestParts<AppState> for Organizer {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let cookie = session_cookie_name(&state.public_url);
        let Some(secret) = cookie_value(&parts.headers, &cookie) else {
            return Err(StatusCode::UNAUTHORIZED);
        };
        let mut connection = state
            .pool
            .acquire()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        session_organizer(&mut connection, secret)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .map(Organizer)
            .ok_or(StatusCode::UNAUTHORIZED)
    }
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(request_sign_in))
        .routes(routes!(sign_in))
        .routes(routes!(current_session, sign_out))
}

#[utoipa::path(
    post,
    path = "/api/sign-in-requests",
    request_body = SignInRequest,
    responses((status = 202, description = "Same answer whether or not the address belongs to an organizer"))
)]
async fn request_sign_in(
    State(state): State<AppState>,
    ClientAddress(client): ClientAddress,
    Json(request): Json<SignInRequest>,
) -> StatusCode {
    if let Some(client) = client
        && !state.sign_in_limiter.allow(client, Instant::now())
    {
        return StatusCode::ACCEPTED;
    }
    if let Err(error) = send_sign_in_link(&state, &request.mail).await {
        tracing::error!(%error, "sign-in link could not be stored");
    }
    StatusCode::ACCEPTED
}

async fn send_sign_in_link(state: &AppState, mail_address: &str) -> Result<(), sqlx::Error> {
    let mut transaction = state.pool.begin().await?;
    let Some(organizer) = members::active_organizer_by_mail(&mut transaction, mail_address).await?
    else {
        return Ok(());
    };
    let recent: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM sign_in_links WHERE member_id = $1 AND created_at > now() - interval '1 hour'",
    )
    .bind(organizer.id)
    .fetch_one(&mut *transaction)
    .await?;
    if recent >= LINKS_PER_ADDRESS_AND_HOUR {
        return Ok(());
    }
    let secret = secret::generate();
    sqlx::query(
        "INSERT INTO sign_in_links (member_id, secret_hash, expires_at) VALUES ($1, $2, now() + interval '1 hour')",
    )
    .bind(organizer.id)
    .bind(secret::hash(&secret))
    .execute(&mut *transaction)
    .await?;
    let link = state.public_url.link(&format!("/sign-in#{secret}"));
    mail::enqueue(&mut transaction, &sign_in_mail(&organizer, &link)).await?;
    transaction.commit().await
}

fn sign_in_mail(organizer: &Member, link: &str) -> Mail {
    let localizer = localization::localizer();
    let mut args = FluentArgs::new();
    args.set("firstName", organizer.first_name.clone());
    let language = organizer.language.as_str();
    let body = localizer.message(language, "sign-in-mail-body", &args);
    let ignore = localizer.message(language, "sign-in-mail-ignore", &args);
    Mail {
        recipient: organizer.mail.clone(),
        member_id: Some(organizer.id),
        invitation_id: None,
        subject: localizer.message(language, "sign-in-mail-subject", &args),
        text: format!("{body}\n\n{link}\n\n{ignore}"),
        html: format!(
            "<p>{}</p><p><a href=\"{link}\">{link}</a></p><p>{}</p>",
            mail::escape_html(&body),
            mail::escape_html(&ignore)
        ),
    }
}

/// Expects the secret of a sign-in link as `Authorization: Bearer <secret>`.
#[utoipa::path(
    post,
    path = "/api/sessions",
    responses(
        (status = 201, description = "Session started, its cookie is set", body = Member),
        (status = 401, description = "The link is unknown, used or expired"),
    )
)]
async fn sign_in(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let Some(link_secret) = secret::from_authorization(&headers) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    match start_session(&state, link_secret).await {
        Ok(Some((organizer, session_secret))) => {
            let cookie = session_cookie(
                &state.public_url,
                &session_secret,
                SESSION_DAYS * 24 * 60 * 60,
            );
            (
                StatusCode::CREATED,
                [(header::SET_COOKIE, cookie)],
                Json(organizer),
            )
                .into_response()
        }
        Ok(None) => StatusCode::UNAUTHORIZED.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// Redeems a sign-in link in one statement, so two clicks cannot both succeed, and starts a session for its organizer.
async fn start_session(
    state: &AppState,
    link_secret: &str,
) -> Result<Option<(Member, String)>, sqlx::Error> {
    let mut transaction = state.pool.begin().await?;
    let member_id: Option<i64> = sqlx::query_scalar(
        "UPDATE sign_in_links SET used_at = now() WHERE secret_hash = $1 AND used_at IS NULL AND expires_at > now() RETURNING member_id",
    )
    .bind(secret::hash(link_secret))
    .fetch_optional(&mut *transaction)
    .await?;
    let Some(member_id) = member_id else {
        return Ok(None);
    };
    let Some(organizer) = members::active_organizer(&mut transaction, member_id).await? else {
        return Ok(None);
    };
    let session_secret = secret::generate();
    sqlx::query("INSERT INTO sessions (member_id, secret_hash, expires_at) VALUES ($1, $2, now() + make_interval(days => $3))")
        .bind(member_id)
        .bind(secret::hash(&session_secret))
        .bind(SESSION_DAYS)
        .execute(&mut *transaction)
        .await?;
    transaction.commit().await?;
    Ok(Some((organizer, session_secret)))
}

#[utoipa::path(
    get,
    path = "/api/session",
    responses(
        (status = 200, description = "The signed-in organizer", body = Member),
        (status = 401, description = "No valid session"),
    )
)]
async fn current_session(Organizer(organizer): Organizer) -> Json<Member> {
    Json(organizer)
}

#[utoipa::path(
    delete,
    path = "/api/session",
    responses((status = 204, description = "Session ended and its cookie removed"))
)]
async fn sign_out(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let name = session_cookie_name(&state.public_url);
    if let Some(secret) = cookie_value(&headers, &name) {
        let deleted = sqlx::query("DELETE FROM sessions WHERE secret_hash = $1")
            .bind(secret::hash(secret))
            .execute(&state.pool)
            .await;
        if deleted.is_err() {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }
    (
        StatusCode::NO_CONTENT,
        [(header::SET_COOKIE, session_cookie(&state.public_url, "", 0))],
    )
        .into_response()
}

async fn session_organizer(
    connection: &mut PgConnection,
    secret: &str,
) -> Result<Option<Member>, sqlx::Error> {
    let member_id: Option<i64> = sqlx::query_scalar(
        "SELECT member_id FROM sessions WHERE secret_hash = $1 AND expires_at > now()",
    )
    .bind(secret::hash(secret))
    .fetch_optional(&mut *connection)
    .await?;
    match member_id {
        Some(member_id) => members::active_organizer(connection, member_id).await,
        None => Ok(None),
    }
}

/// Browsers refuse a cookie with the prefix `__Host-` from any other host. On `localhost` they ignore the port, so the name carries it and instances side by side keep separate sessions.
fn session_cookie_name(public_url: &PublicUrl) -> String {
    match public_url.localhost_port() {
        Some(port) => format!("adunata_session_{port}"),
        None => SECURE_COOKIE_NAME.to_owned(),
    }
}

fn session_cookie(public_url: &PublicUrl, value: &str, max_age_seconds: i32) -> HeaderValue {
    let secure = if public_url.localhost_port().is_some() {
        ""
    } else {
        "; Secure"
    };
    let cookie = format!(
        "{}={value}; Max-Age={max_age_seconds}; Path=/; HttpOnly; SameSite=Strict{secure}",
        session_cookie_name(public_url)
    );
    HeaderValue::from_str(&cookie).expect("cookie holds only hexadecimal characters and fixed text")
}

fn cookie_value<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(';'))
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(key, _)| *key == name)
        .map(|(_, value)| value)
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
pub mod testing {
    use sqlx::PgPool;

    use super::SESSION_DAYS;
    use crate::secret;

    /// Starts a session for a member and returns the `Cookie` header that carries it for an instance at `https://adunata.example.org`.
    pub async fn session_cookie(pool: &PgPool, member_id: i64) -> String {
        let secret = secret::generate();
        sqlx::query("INSERT INTO sessions (member_id, secret_hash, expires_at) VALUES ($1, $2, now() + make_interval(days => $3))")
            .bind(member_id)
            .bind(secret::hash(&secret))
            .bind(SESSION_DAYS)
            .execute(pool)
            .await
            .unwrap();
        format!("{}={secret}", super::SECURE_COOKIE_NAME)
    }

    /// Stores an active organizer named Root Organizer and returns the member's id with a session cookie.
    pub async fn signed_in_organizer(pool: &PgPool) -> (i64, String) {
        let id: i64 = sqlx::query_scalar(
            "INSERT INTO members (first_name, last_name, mail, language, organizer) VALUES ('Root', 'Organizer', 'root@example.org', 'en', true) RETURNING id",
        )
        .fetch_one(pool)
        .await
        .unwrap();
        (id, session_cookie(pool, id).await)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn origin_of_the_public_url() {
        let origin = |url| PublicUrl::parse(url).unwrap().origin();
        assert_eq!(
            origin("https://choir.example.org/adunata/"),
            "https://choir.example.org"
        );
        assert_eq!(origin("http://localhost:8081"), "http://localhost:8081");
        assert_eq!(
            origin("https://Choir.Example.org:443/"),
            "https://choir.example.org"
        );
        assert_eq!(origin("http://localhost:80"), "http://localhost");
    }

    mod sign_in {
        use super::*;

        #[test]
        fn client_address_behind_proxies() {
            let trusted = [Network::parse("10.0.0.0/8").unwrap()];
            let headers = |value: &str| {
                let mut headers = HeaderMap::new();
                headers.insert("x-forwarded-for", value.parse().unwrap());
                headers
            };
            let proxy: IpAddr = "10.0.0.2".parse().unwrap();
            assert_eq!(
                client_address(proxy, &headers("1.1.1.1, 2.2.2.2, 10.0.0.3"), &trusted),
                "2.2.2.2".parse::<IpAddr>().unwrap()
            );
            assert_eq!(client_address(proxy, &HeaderMap::new(), &trusted), proxy);
            assert_eq!(client_address(proxy, &headers("garbage"), &trusted), proxy);
        }

        #[test]
        fn limit_frees_up_after_an_hour() {
            let limiter = SignInLimiter::default();
            let address: IpAddr = "203.0.113.1".parse().unwrap();
            let start = Instant::now();
            for _ in 0..20 {
                assert!(limiter.allow(address, start));
            }
            assert!(!limiter.allow(address, start + Duration::from_mins(59)));
            assert!(limiter.allow(address, start + HOUR));
        }

        mod integration {
            use super::*;
            use crate::api;
            use crate::configuration::FirstOrganizer;
            use crate::database::testing::fresh_database;
            use axum::body::{Body, to_bytes};
            use axum::http::Response as HttpResponse;
            use serde_json::{Value, json};
            use sqlx::PgPool;
            use tower::ServiceExt;

            async fn call(state: AppState, request: Request<Body>) -> HttpResponse<Body> {
                api::router(state).oneshot(request).await.unwrap()
            }

            async fn organizer(pool: &PgPool) -> i64 {
                sqlx::query_scalar(
                            "INSERT INTO members (first_name, last_name, mail, language, organizer) VALUES ('Ada', 'Lovelace', 'ada@example.org', 'de', true) RETURNING id",
                        )
                        .fetch_one(pool)
                        .await
                        .unwrap()
            }

            async fn request_link(pool: &PgPool, mail: &str) -> StatusCode {
                let request = Request::post("/api/sign-in-requests")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(json!({ "mail": mail }).to_string()))
                    .unwrap();
                call(api::testing::state(pool.clone()), request)
                    .await
                    .status()
            }

            /// The secret of the newest sign-in link, read from the mail in the outbox.
            async fn link_secret(pool: &PgPool) -> String {
                let text: String = sqlx::query_scalar(
                    "SELECT text_body FROM mail_outbox ORDER BY id DESC LIMIT 1",
                )
                .fetch_one(pool)
                .await
                .unwrap();
                let start =
                    text.find("/sign-in#").expect("mail must contain the link") + "/sign-in#".len();
                text[start..start + 64].to_owned()
            }

            async fn redeem(state: AppState, secret: &str) -> HttpResponse<Body> {
                let request = Request::post("/api/sessions")
                    .header(header::AUTHORIZATION, format!("Bearer {secret}"))
                    .body(Body::empty())
                    .unwrap();
                call(state, request).await
            }

            fn cookie_of(response: &HttpResponse<Body>) -> String {
                let set_cookie = response.headers()[header::SET_COOKIE].to_str().unwrap();
                set_cookie.split(';').next().unwrap().to_owned()
            }

            async fn session_status(state: AppState, cookie: &str) -> StatusCode {
                let request = Request::get("/api/session")
                    .header(header::COOKIE, cookie)
                    .body(Body::empty())
                    .unwrap();
                call(state, request).await.status()
            }

            async fn count(pool: &PgPool, query: &str) -> i64 {
                sqlx::query_scalar(sqlx::AssertSqlSafe(query.to_owned()))
                    .fetch_one(pool)
                    .await
                    .unwrap()
            }

            #[tokio::test]
            async fn successful_sign_in() {
                let database = fresh_database().await;
                let pool = database.pool();
                organizer(&pool).await;
                assert_eq!(
                    request_link(&pool, "ADA@example.org").await,
                    StatusCode::ACCEPTED
                );
                let secret = link_secret(&pool).await;
                let text: String = sqlx::query_scalar("SELECT text_body FROM mail_outbox")
                    .fetch_one(&pool)
                    .await
                    .unwrap();
                assert!(text.starts_with("Hallo Ada"), "{text}");

                let state = api::testing::state(pool.clone());
                let response = redeem(state.clone(), &secret).await;
                assert_eq!(response.status(), StatusCode::CREATED);
                let set_cookie = response.headers()[header::SET_COOKIE]
                    .to_str()
                    .unwrap()
                    .to_owned();
                assert!(
                    set_cookie.starts_with("__Host-adunata_session="),
                    "{set_cookie}"
                );
                for attribute in ["Secure", "HttpOnly", "SameSite=Strict", "Path=/"] {
                    assert!(set_cookie.contains(attribute), "{set_cookie}");
                }
                let cookie = cookie_of(&response);
                let body: Value = serde_json::from_slice(
                    &to_bytes(response.into_body(), usize::MAX).await.unwrap(),
                )
                .unwrap();
                assert_eq!(body["firstName"], "Ada");
                assert_eq!(session_status(state, &cookie).await, StatusCode::OK);
            }

            #[tokio::test]
            async fn link_used_twice() {
                let database = fresh_database().await;
                let pool = database.pool();
                organizer(&pool).await;
                request_link(&pool, "ada@example.org").await;
                let secret = link_secret(&pool).await;
                let state = api::testing::state(pool.clone());

                assert_eq!(
                    redeem(state.clone(), &secret).await.status(),
                    StatusCode::CREATED
                );
                assert_eq!(
                    redeem(state, &secret).await.status(),
                    StatusCode::UNAUTHORIZED
                );
                assert_eq!(count(&pool, "SELECT count(*) FROM sessions").await, 1);
            }

            #[tokio::test]
            async fn link_opened_by_a_mail_scanner() {
                let database = fresh_database().await;
                let pool = database.pool();
                organizer(&pool).await;
                request_link(&pool, "ada@example.org").await;
                let secret = link_secret(&pool).await;

                let page = crate::client::router()
                    .oneshot(Request::get("/sign-in").body(Body::empty()).unwrap())
                    .await
                    .unwrap();
                assert_eq!(page.status(), StatusCode::OK);
                assert!(!page.headers().contains_key(header::SET_COOKIE));
                assert_eq!(count(&pool, "SELECT count(*) FROM sessions").await, 0);
                assert_eq!(
                    count(
                        &pool,
                        "SELECT count(*) FROM sign_in_links WHERE used_at IS NULL"
                    )
                    .await,
                    1
                );
                assert_eq!(
                    redeem(api::testing::state(pool.clone()), &secret)
                        .await
                        .status(),
                    StatusCode::CREATED
                );
            }

            #[tokio::test]
            async fn expired_link() {
                let database = fresh_database().await;
                let pool = database.pool();
                organizer(&pool).await;
                request_link(&pool, "ada@example.org").await;
                let secret = link_secret(&pool).await;
                sqlx::query("UPDATE sign_in_links SET expires_at = now() - interval '1 second'")
                    .execute(&pool)
                    .await
                    .unwrap();
                assert_eq!(
                    redeem(api::testing::state(pool.clone()), &secret)
                        .await
                        .status(),
                    StatusCode::UNAUTHORIZED
                );
                assert_eq!(count(&pool, "SELECT count(*) FROM sessions").await, 0);
            }

            #[tokio::test]
            async fn link_expires_one_hour_after_sending() {
                let database = fresh_database().await;
                let pool = database.pool();
                organizer(&pool).await;
                request_link(&pool, "ada@example.org").await;
                let seconds: f64 = sqlx::query_scalar(
                    "SELECT EXTRACT(EPOCH FROM expires_at - created_at)::float8 FROM sign_in_links",
                )
                .fetch_one(&pool)
                .await
                .unwrap();
                assert!((seconds - 3600.0).abs() < 1.0, "{seconds}");
            }

            #[tokio::test]
            async fn unknown_address() {
                let database = fresh_database().await;
                let pool = database.pool();
                organizer(&pool).await;
                sqlx::query("INSERT INTO members (first_name, last_name, mail, language) VALUES ('Clara', 'Schumann', 'clara@example.org', 'de')")
                            .execute(&pool)
                            .await
                            .unwrap();
                for mail in ["nobody@example.org", "clara@example.org"] {
                    assert_eq!(request_link(&pool, mail).await, StatusCode::ACCEPTED);
                }
                assert_eq!(count(&pool, "SELECT count(*) FROM mail_outbox").await, 0);
                assert_eq!(count(&pool, "SELECT count(*) FROM sign_in_links").await, 0);
            }

            async fn signed_in(pool: &PgPool) -> (AppState, String) {
                let id = organizer(pool).await;
                (
                    api::testing::state(pool.clone()),
                    crate::access::testing::session_cookie(pool, id).await,
                )
            }

            #[tokio::test]
            async fn sign_out() {
                let database = fresh_database().await;
                let pool = database.pool();
                let (state, cookie) = signed_in(&pool).await;
                let request = Request::delete("/api/session")
                    .header(header::COOKIE, &cookie)
                    .body(Body::empty())
                    .unwrap();
                let response = call(state.clone(), request).await;
                assert_eq!(response.status(), StatusCode::NO_CONTENT);
                assert!(
                    response.headers()[header::SET_COOKIE]
                        .to_str()
                        .unwrap()
                        .contains("Max-Age=0")
                );
                assert_eq!(
                    session_status(state, &cookie).await,
                    StatusCode::UNAUTHORIZED
                );
                assert_eq!(count(&pool, "SELECT count(*) FROM sessions").await, 0);
            }

            #[tokio::test]
            async fn session_expires() {
                let database = fresh_database().await;
                let pool = database.pool();
                let (state, cookie) = signed_in(&pool).await;
                sqlx::query("UPDATE sessions SET created_at = now() - interval '14 days 1 minute', expires_at = now() - interval '1 minute'")
                            .execute(&pool)
                            .await
                            .unwrap();
                assert_eq!(
                    session_status(state, &cookie).await,
                    StatusCode::UNAUTHORIZED
                );
            }

            #[tokio::test]
            async fn session_lasts_14_days() {
                let database = fresh_database().await;
                let pool = database.pool();
                organizer(&pool).await;
                request_link(&pool, "ada@example.org").await;
                let response =
                    redeem(api::testing::state(pool.clone()), &link_secret(&pool).await).await;
                assert!(
                    response.headers()[header::SET_COOKIE]
                        .to_str()
                        .unwrap()
                        .contains("Max-Age=1209600")
                );
                let days: f64 = sqlx::query_scalar(
                            "SELECT EXTRACT(EPOCH FROM expires_at - created_at)::float8 / 86400 FROM sessions",
                        )
                        .fetch_one(&pool)
                        .await
                        .unwrap();
                assert!((days - 14.0).abs() < 0.001, "{days}");
            }

            #[tokio::test]
            async fn role_revoked() {
                let database = fresh_database().await;
                let pool = database.pool();
                let (state, cookie) = signed_in(&pool).await;
                assert_eq!(session_status(state.clone(), &cookie).await, StatusCode::OK);
                sqlx::query("UPDATE members SET organizer = false")
                    .execute(&pool)
                    .await
                    .unwrap();
                assert_eq!(
                    session_status(state, &cookie).await,
                    StatusCode::UNAUTHORIZED
                );
            }

            #[tokio::test]
            async fn deactivated_organizer_loses_the_session() {
                let database = fresh_database().await;
                let pool = database.pool();
                let (state, cookie) = signed_in(&pool).await;
                sqlx::query("UPDATE members SET active = false")
                    .execute(&pool)
                    .await
                    .unwrap();
                assert_eq!(
                    session_status(state, &cookie).await,
                    StatusCode::UNAUTHORIZED
                );
            }

            async fn request_from(
                state: &AppState,
                peer: &str,
                forwarded: Option<&str>,
                mail: &str,
            ) -> StatusCode {
                let mut request = Request::post("/api/sign-in-requests")
                    .header(header::CONTENT_TYPE, "application/json");
                if let Some(forwarded) = forwarded {
                    request = request.header("x-forwarded-for", forwarded);
                }
                let mut request = request
                    .body(Body::from(json!({ "mail": mail }).to_string()))
                    .unwrap();
                request
                    .extensions_mut()
                    .insert(ConnectInfo(SocketAddr::new(peer.parse().unwrap(), 40000)));
                call(state.clone(), request).await.status()
            }

            async fn links(pool: &PgPool) -> i64 {
                count(pool, "SELECT count(*) FROM sign_in_links").await
            }

            #[tokio::test]
            async fn repeated_requests() {
                let database = fresh_database().await;
                let pool = database.pool();
                organizer(&pool).await;
                for _ in 0..4 {
                    assert_eq!(
                        request_link(&pool, "ada@example.org").await,
                        StatusCode::ACCEPTED
                    );
                }
                assert_eq!(links(&pool).await, 3);
                assert_eq!(count(&pool, "SELECT count(*) FROM mail_outbox").await, 3);
            }

            #[tokio::test]
            async fn many_requests_from_one_network_address() {
                let database = fresh_database().await;
                let pool = database.pool();
                organizer(&pool).await;
                let state = api::testing::state(pool.clone());
                for number in 0..20 {
                    let mail = format!("guess{number}@example.org");
                    assert_eq!(
                        request_from(&state, "203.0.113.7", None, &mail).await,
                        StatusCode::ACCEPTED
                    );
                }
                assert_eq!(
                    request_from(&state, "203.0.113.7", None, "ada@example.org").await,
                    StatusCode::ACCEPTED
                );
                assert_eq!(links(&pool).await, 0);
                request_from(&state, "203.0.113.8", None, "ada@example.org").await;
                assert_eq!(links(&pool).await, 1);
            }

            #[tokio::test]
            async fn request_through_a_trusted_proxy() {
                let database = fresh_database().await;
                let pool = database.pool();
                organizer(&pool).await;
                let state = api::testing::state(pool.clone());
                for _ in 0..20 {
                    request_from(
                        &state,
                        "127.0.0.1",
                        Some("198.51.100.1"),
                        "nobody@example.org",
                    )
                    .await;
                }
                request_from(&state, "127.0.0.1", Some("198.51.100.1"), "ada@example.org").await;
                assert_eq!(links(&pool).await, 0);
                request_from(
                    &state,
                    "127.0.0.1",
                    Some("6.6.6.6, 198.51.100.2"),
                    "ada@example.org",
                )
                .await;
                assert_eq!(links(&pool).await, 1);
            }

            #[tokio::test]
            async fn header_from_an_untrusted_client() {
                let database = fresh_database().await;
                let pool = database.pool();
                organizer(&pool).await;
                let state = api::testing::state(pool.clone());
                for number in 0..20 {
                    let forwarded = format!("198.51.100.{number}");
                    request_from(
                        &state,
                        "203.0.113.9",
                        Some(&forwarded),
                        "nobody@example.org",
                    )
                    .await;
                }
                request_from(
                    &state,
                    "203.0.113.9",
                    Some("198.51.100.200"),
                    "ada@example.org",
                )
                .await;
                assert_eq!(links(&pool).await, 0);
            }

            fn ada() -> FirstOrganizer {
                FirstOrganizer {
                    mail: "ada@example.org".to_owned(),
                    first_name: "Ada".to_owned(),
                    last_name: "Lovelace".to_owned(),
                }
            }

            async fn organizers(pool: &PgPool) -> Vec<(String, String)> {
                sqlx::query_as(
                    "SELECT mail, language FROM members WHERE active AND organizer ORDER BY id",
                )
                .fetch_all(pool)
                .await
                .unwrap()
            }

            #[tokio::test]
            async fn set_up_a_new_instance() {
                let database = fresh_database().await;
                let pool = database.pool();
                let outcome = ensure_first_organizer(&pool, Some(&ada()), "de")
                    .await
                    .unwrap();
                assert_eq!(outcome, FirstOrganizerOutcome::Created);
                assert_eq!(
                    organizers(&pool).await,
                    vec![("ada@example.org".to_owned(), "de".to_owned())]
                );
            }

            #[tokio::test]
            async fn instance_without_any_organizer() {
                let database = fresh_database().await;
                let pool = database.pool();
                let outcome = ensure_first_organizer(&pool, None, "de").await.unwrap();
                assert_eq!(outcome, FirstOrganizerOutcome::NobodyCanSignIn);
                assert!(organizers(&pool).await.is_empty());
            }

            #[tokio::test]
            async fn instance_already_has_an_organizer() {
                let database = fresh_database().await;
                let pool = database.pool();
                sqlx::query("INSERT INTO members (first_name, last_name, mail, language, organizer) VALUES ('Clara', 'Schumann', 'clara@example.org', 'en', true)")
                            .execute(&pool)
                            .await
                            .unwrap();
                let outcome = ensure_first_organizer(&pool, Some(&ada()), "de")
                    .await
                    .unwrap();
                assert_eq!(outcome, FirstOrganizerOutcome::AlreadyPresent);
                assert_eq!(
                    organizers(&pool).await,
                    vec![("clara@example.org".to_owned(), "en".to_owned())]
                );
                assert_eq!(count(&pool, "SELECT count(*) FROM members").await, 1);
            }

            #[tokio::test]
            async fn every_organizer_mailbox_lost() {
                let database = fresh_database().await;
                let pool = database.pool();
                sqlx::query("INSERT INTO members (first_name, last_name, mail, language, active, organizer) VALUES ('Clara', 'Schumann', 'clara@example.org', 'en', false, true)")
                            .execute(&pool)
                            .await
                            .unwrap();
                let mut connection = pool.acquire().await.unwrap();
                members::grant_organizer(&mut connection, "CLARA@example.org", "C", "S", "de")
                    .await
                    .unwrap();
                members::grant_organizer(
                    &mut connection,
                    "new@example.org",
                    "New",
                    "Organizer",
                    "de",
                )
                .await
                .unwrap();
                drop(connection);
                assert_eq!(
                    organizers(&pool).await,
                    vec![
                        ("clara@example.org".to_owned(), "en".to_owned()),
                        ("new@example.org".to_owned(), "de".to_owned())
                    ]
                );
                request_link(&pool, "new@example.org").await;
                assert_eq!(links(&pool).await, 1);
            }

            #[tokio::test]
            async fn instances_on_localhost_keep_separate_sessions() {
                let database = fresh_database().await;
                let pool = database.pool();
                organizer(&pool).await;
                request_link(&pool, "ada@example.org").await;
                let first = AppState {
                    public_url: PublicUrl::parse("http://localhost:8081").unwrap(),
                    ..api::testing::state(pool.clone())
                };
                let second = AppState {
                    public_url: PublicUrl::parse("http://localhost:8082").unwrap(),
                    ..api::testing::state(pool.clone())
                };
                let response = redeem(first.clone(), &link_secret(&pool).await).await;
                let set_cookie = response.headers()[header::SET_COOKIE]
                    .to_str()
                    .unwrap()
                    .to_owned();
                assert!(
                    set_cookie.starts_with("adunata_session_8081="),
                    "{set_cookie}"
                );
                assert!(!set_cookie.contains("Secure"), "{set_cookie}");
                let cookie = cookie_of(&response);
                assert_eq!(session_status(first, &cookie).await, StatusCode::OK);
                assert_eq!(
                    session_status(second, &cookie).await,
                    StatusCode::UNAUTHORIZED
                );
            }
        }
    }

    mod integration {
        use super::*;
        use axum::body::Body;
        use axum::http::Request;
        use axum::routing::post;
        use tower::ServiceExt;

        fn app() -> Router {
            let public_url = PublicUrl::parse("https://adunata.example.org/").unwrap();
            with_origin_check(
                Router::new().route(
                    "/api/things",
                    post(|| async { "changed" }).get(|| async { "read" }),
                ),
                &public_url,
            )
        }

        async fn status(method: &str, origin: Option<&str>) -> StatusCode {
            let mut request = Request::builder().method(method).uri("/api/things");
            if let Some(origin) = origin {
                request = request.header(header::ORIGIN, origin);
            }
            app()
                .oneshot(request.body(Body::empty()).unwrap())
                .await
                .unwrap()
                .status()
        }

        #[tokio::test]
        async fn request_from_another_origin() {
            assert_eq!(
                status("POST", Some("https://evil.example.org")).await,
                StatusCode::FORBIDDEN
            );
            assert_eq!(
                status("POST", Some("https://sub.adunata.example.org")).await,
                StatusCode::FORBIDDEN
            );
            assert_eq!(status("POST", Some("null")).await, StatusCode::FORBIDDEN);
        }

        #[tokio::test]
        async fn request_from_the_public_origin() {
            assert_eq!(
                status("POST", Some("https://adunata.example.org")).await,
                StatusCode::OK
            );
        }

        #[tokio::test]
        async fn request_without_an_origin() {
            assert_eq!(status("POST", None).await, StatusCode::OK);
        }

        #[tokio::test]
        async fn reading_from_another_origin() {
            assert_eq!(
                status("GET", Some("https://evil.example.org")).await,
                StatusCode::OK
            );
        }
    }
}
