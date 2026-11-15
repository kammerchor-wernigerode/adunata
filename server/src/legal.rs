use axum::Json;
use axum::extract::State;
use serde::Serialize;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::api::AppState;

/// Addresses of the privacy notice and the imprint that every page links. A missing address means the operator set none.
#[derive(Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LegalLinks {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub privacy_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub imprint_url: Option<String>,
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(legal_links))
}

#[utoipa::path(
    get,
    path = "/api/legal-links",
    responses((status = 200, description = "Links every page shows", body = LegalLinks))
)]
async fn legal_links(State(state): State<AppState>) -> Json<LegalLinks> {
    Json(LegalLinks {
        privacy_url: state.legal_links.privacy.clone(),
        imprint_url: state.legal_links.imprint.clone(),
    })
}

#[cfg(test)]
mod tests {
    use axum::body::{Body, to_bytes};
    use axum::http::Request;
    use serde_json::{Value, json};
    use tower::ServiceExt;

    use crate::api;
    use crate::configuration::LegalLinks as Configured;

    async fn get(legal_links: Configured) -> Value {
        let pool = sqlx::PgPool::connect_lazy("postgres://localhost").unwrap();
        let state = api::AppState {
            legal_links,
            ..api::testing::state(pool)
        };
        let response = api::router(state)
            .oneshot(
                Request::get("/api/legal-links")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap()).unwrap()
    }

    #[tokio::test]
    async fn both_addresses_set() {
        let body = get(Configured {
            privacy: Some("https://choir.example.org/privacy".to_owned()),
            imprint: Some("https://choir.example.org/imprint".to_owned()),
        })
        .await;
        assert_eq!(
            body,
            json!({
                "privacyUrl": "https://choir.example.org/privacy",
                "imprintUrl": "https://choir.example.org/imprint"
            })
        );
    }

    #[tokio::test]
    async fn no_address_set() {
        assert_eq!(get(Configured::default()).await, json!({}));
    }
}
