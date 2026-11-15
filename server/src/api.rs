use axum::Router;
use sqlx::PgPool;
use utoipa::OpenApi;
use utoipa::openapi::OpenApi as Document;
use utoipa_axum::router::OpenApiRouter;

use crate::configuration::{LegalLinks, PublicUrl};
use crate::{access, health, legal, localization, members, serving, spike};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub public_url: PublicUrl,
    pub default_language: String,
    pub legal_links: LegalLinks,
}

#[derive(OpenApi)]
#[openapi(info(title = "Adunata"))]
struct ApiDoc;

fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::with_openapi(ApiDoc::openapi())
        .merge(health::router())
        .merge(spike::router())
        .merge(localization::router())
        .merge(legal::router())
        .merge(members::router())
        .merge(access::router())
}

pub fn router(state: AppState) -> Router {
    let public_url = state.public_url.clone();
    let api = routes().split_for_parts().0.with_state(state);
    serving::without_caching(access::with_origin_check(api, &public_url))
}

pub fn document() -> Document {
    routes().split_for_parts().1
}

#[cfg(test)]
pub mod testing {
    use sqlx::PgPool;

    use super::AppState;
    use crate::configuration::{LegalLinks, PublicUrl};

    pub const PUBLIC_URL: &str = "https://adunata.example.org";

    pub fn state(pool: PgPool) -> AppState {
        AppState {
            pool,
            public_url: PublicUrl::parse(PUBLIC_URL).expect("test URL must be valid"),
            default_language: "de".to_owned(),
            legal_links: LegalLinks::default(),
        }
    }
}
