use axum::Router;
use sqlx::PgPool;
use utoipa::OpenApi;
use utoipa::openapi::OpenApi as Document;
use utoipa_axum::router::OpenApiRouter;

use crate::spike;

#[derive(OpenApi)]
#[openapi(info(title = "Adunata"))]
struct ApiDoc;

fn routes() -> OpenApiRouter<PgPool> {
    OpenApiRouter::with_openapi(ApiDoc::openapi()).merge(spike::router())
}

pub fn router(pool: PgPool) -> Router {
    routes().split_for_parts().0.with_state(pool)
}

pub fn document() -> Document {
    routes().split_for_parts().1
}
