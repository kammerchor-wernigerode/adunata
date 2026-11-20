use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::time::Duration;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use serde::Serialize;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::api::AppState;

#[derive(Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Ready,
    Unavailable,
}

#[derive(Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum Cause {
    Database,
}

/// Whether the instance can serve requests. It holds no personal data and no configuration values.
#[derive(Debug, PartialEq, Eq, Serialize, ToSchema)]
pub struct Health {
    pub status: Status,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cause: Option<Cause>,
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(health))
}

#[utoipa::path(
    get,
    path = "/api/health",
    responses(
        (status = 200, description = "Ready to serve requests", body = Health),
        (status = 503, description = "A dependency is unavailable", body = Health),
    )
)]
async fn health(State(state): State<AppState>) -> (StatusCode, Json<Health>) {
    let database_reachable = sqlx::query("SELECT 1").execute(&state.pool).await.is_ok();
    if database_reachable {
        (
            StatusCode::OK,
            Json(Health {
                status: Status::Ready,
                cause: None,
            }),
        )
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(Health {
                status: Status::Unavailable,
                cause: Some(Cause::Database),
            }),
        )
    }
}

/// Asks the instance on a listen address whether it is ready. Fails with one line that names the cause and nothing from the answer body.
pub async fn check_readiness(listen_address: SocketAddr, deadline: Duration) -> Result<(), String> {
    tokio::time::timeout(
        deadline,
        ask_for_readiness(reachable_address(listen_address)),
    )
    .await
    .unwrap_or_else(|_| {
        Err(format!(
            "the instance did not answer within {} seconds",
            deadline.as_secs_f64()
        ))
    })
}

async fn ask_for_readiness(address: SocketAddr) -> Result<(), String> {
    let mut connection = TcpStream::connect(address)
        .await
        .map_err(|error| format!("the instance cannot be reached: {error}"))?;
    let mut status_line = String::new();
    connection
        .write_all(b"GET /api/health HTTP/1.0\r\n\r\n")
        .await
        .map_err(|error| format!("the connection to the instance failed: {error}"))?;
    BufReader::new(connection)
        .take(256)
        .read_line(&mut status_line)
        .await
        .map_err(|error| format!("the connection to the instance failed: {error}"))?;
    match status_code(&status_line) {
        Some(200) => Ok(()),
        Some(code) => Err(format!("the instance answered with status {code}")),
        None => Err("the instance sent no HTTP answer".to_owned()),
    }
}

/// The status code of an HTTP status line such as `HTTP/1.0 200 OK`.
fn status_code(status_line: &str) -> Option<u16> {
    status_line
        .strip_prefix("HTTP/")?
        .split_whitespace()
        .nth(1)?
        .parse()
        .ok()
}

/// The address to ask for a listen address, which is the loopback address of the same family when the instance listens on every interface.
fn reachable_address(mut listen_address: SocketAddr) -> SocketAddr {
    match listen_address.ip() {
        IpAddr::V4(ip) if ip.is_unspecified() => listen_address.set_ip(Ipv4Addr::LOCALHOST.into()),
        IpAddr::V6(ip) if ip.is_unspecified() => listen_address.set_ip(Ipv6Addr::LOCALHOST.into()),
        _ => {}
    }
    listen_address
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_ipv4_interface_means_loopback() {
        let address = reachable_address("0.0.0.0:8080".parse().unwrap());
        assert_eq!(address, "127.0.0.1:8080".parse().unwrap());
    }

    #[test]
    fn every_ipv6_interface_means_loopback() {
        let address = reachable_address("[::]:8080".parse().unwrap());
        assert_eq!(address, "[::1]:8080".parse().unwrap());
    }

    #[test]
    fn specific_address_stays() {
        let address = reachable_address("192.0.2.1:8080".parse().unwrap());
        assert_eq!(address, "192.0.2.1:8080".parse().unwrap());
    }

    #[test]
    fn status_code_of_an_http_answer() {
        assert_eq!(
            status_code("HTTP/1.0 503 Service Unavailable\r\n"),
            Some(503)
        );
    }

    #[test]
    fn answer_without_http_has_no_status_code() {
        assert_eq!(status_code("not-http 200\r\n"), None);
    }

    #[test]
    fn link_local_address_keeps_its_scope() {
        let address = reachable_address("[fe80::1%2]:8080".parse().unwrap());
        assert_eq!(address, "[fe80::1%2]:8080".parse().unwrap());
    }
}

#[cfg(test)]
mod integration {
    use std::time::Duration;

    use axum::body::{Body, to_bytes};
    use axum::http::Request;
    use serde_json::{Value, json};
    use sqlx::postgres::PgPoolOptions;
    use tower::ServiceExt;

    use super::*;
    use crate::api;
    use crate::database::testing::fresh_database;
    use crate::serving::serve_until;

    fn unreachable_pool() -> sqlx::PgPool {
        PgPoolOptions::new()
            .acquire_timeout(Duration::from_millis(500))
            .connect_lazy("postgres://adunata@127.0.0.1:1/adunata")
            .unwrap()
    }

    async fn check_served(pool: sqlx::PgPool) -> Result<(), String> {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let (stop, stopped) = tokio::sync::oneshot::channel();
        let server = tokio::spawn(serve_until(
            listener,
            api::router(api::testing::state(pool)),
            stopped,
        ));
        let outcome = check_readiness(address, Duration::from_secs(10)).await;
        stop.send(()).unwrap();
        server.await.unwrap().unwrap();
        outcome
    }

    async fn get_health(pool: sqlx::PgPool) -> (StatusCode, Value) {
        let response = api::router(api::testing::state(pool))
            .oneshot(Request::get("/api/health").body(Body::empty()).unwrap())
            .await
            .unwrap();
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        (status, serde_json::from_slice(&body).unwrap())
    }

    #[tokio::test]
    async fn healthy_instance() {
        let database = fresh_database().await;
        let (status, body) = get_health(database.pool()).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, json!({ "status": "ready" }));
    }

    #[tokio::test]
    async fn database_unreachable() {
        let (status, body) = get_health(unreachable_pool()).await;
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(
            body,
            json!({ "status": "unavailable", "cause": "database" })
        );
    }

    #[tokio::test]
    async fn check_of_a_ready_instance_succeeds() {
        let database = fresh_database().await;
        assert_eq!(check_served(database.pool()).await, Ok(()));
    }

    #[tokio::test]
    async fn check_names_the_status_and_nothing_from_the_body() {
        let cause = check_served(unreachable_pool()).await.unwrap_err();
        assert_eq!(cause, "the instance answered with status 503");
    }

    #[tokio::test]
    async fn check_gives_up_on_a_silent_instance() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let silent = tokio::spawn(async move {
            let (_connection, _) = listener.accept().await.unwrap();
            std::future::pending::<()>().await;
        });
        let cause = check_readiness(address, Duration::from_millis(200))
            .await
            .unwrap_err();
        silent.abort();
        assert_eq!(cause, "the instance did not answer within 0.2 seconds");
    }
}
