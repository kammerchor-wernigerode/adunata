use axum::Router;
use axum::http::{StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use rust_embed::Embed;

const INDEX: &str = "index.html";

#[derive(Embed)]
#[folder = "../client/dist"]
struct Assets;

pub fn router() -> Router {
    Router::new().fallback(serve)
}

async fn serve(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    asset(path)
        .or_else(|| is_client_route(path).then(|| asset(INDEX)).flatten())
        .unwrap_or_else(|| StatusCode::NOT_FOUND.into_response())
}

fn asset(path: &str) -> Option<Response> {
    let path = if path.is_empty() { INDEX } else { path };
    let file = Assets::get(path)?;
    let mime = file.metadata.mimetype().to_owned();
    Some(([(header::CONTENT_TYPE, mime)], file.data).into_response())
}

fn is_client_route(path: &str) -> bool {
    !path.rsplit('/').next().unwrap_or_default().contains('.')
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    async fn get(path: &str) -> Response {
        router()
            .oneshot(Request::get(path).body(Body::empty()).unwrap())
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn root_serves_the_client() {
        let response = get("/").await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[header::CONTENT_TYPE], "text/html");
    }

    #[tokio::test]
    async fn client_route_serves_the_client() {
        let response = get("/gatherings/42").await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[header::CONTENT_TYPE], "text/html");
    }

    #[tokio::test]
    async fn missing_asset_is_not_found() {
        let response = get("/assets/missing.js").await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
