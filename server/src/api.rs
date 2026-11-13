use axum::Router;
use utoipa::OpenApi;
use utoipa::openapi::OpenApi as Document;
use utoipa_axum::router::OpenApiRouter;

use crate::spike;

#[derive(OpenApi)]
#[openapi(info(title = "Adunata"))]
struct ApiDoc;

fn split() -> (Router, Document) {
    OpenApiRouter::with_openapi(ApiDoc::openapi())
        .merge(spike::router())
        .split_for_parts()
}

pub fn router() -> Router {
    split().0
}

pub fn document() -> Document {
    split().1
}
