use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use serde::Serialize;
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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use axum::body::{Body, to_bytes};
    use axum::http::Request;
    use serde_json::{Value, json};
    use sqlx::postgres::PgPoolOptions;
    use tower::ServiceExt;

    use super::*;
    use crate::api;
    use crate::database::testing::fresh_database;

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
        let (status, body) = get_health(fresh_database().await).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, json!({ "status": "ready" }));
    }

    #[tokio::test]
    async fn database_unreachable() {
        let pool = PgPoolOptions::new()
            .acquire_timeout(Duration::from_millis(500))
            .connect_lazy("postgres://adunata@127.0.0.1:1/adunata")
            .unwrap();
        let (status, body) = get_health(pool).await;
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(
            body,
            json!({ "status": "unavailable", "cause": "database" })
        );
    }
}
