use axum::Router;
use axum::extract::{Request, State};
use axum::http::{Method, StatusCode, header};
use axum::middleware::{Next, from_fn_with_state};
use axum::response::{IntoResponse, Response};

use crate::configuration::PublicUrl;

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

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::Request;
    use axum::routing::post;
    use tower::ServiceExt;

    use super::*;

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

    #[test]
    fn origin_of_the_public_url() {
        let origin = |url| PublicUrl::parse(url).unwrap().origin();
        assert_eq!(
            origin("https://choir.example.org/adunata/"),
            "https://choir.example.org"
        );
        assert_eq!(origin("http://localhost:8081"), "http://localhost:8081");
    }
}
