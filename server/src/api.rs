use axum::Router;
use sqlx::PgPool;
use utoipa::OpenApi;
use utoipa::openapi::OpenApi as Document;
use utoipa_axum::router::OpenApiRouter;

use crate::access::SignInLimiter;
use crate::configuration::{InstanceTimeZone, LegalLinks, Network, PublicUrl};
use crate::{access, gatherings, health, legal, localization, members, serving, spike};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub public_url: PublicUrl,
    pub default_language: String,
    pub legal_links: LegalLinks,
    pub trusted_proxies: Vec<Network>,
    pub sign_in_limiter: SignInLimiter,
    pub time_zone: InstanceTimeZone,
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
        .merge(gatherings::router())
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
    use crate::access::SignInLimiter;
    use crate::configuration::{InstanceTimeZone, LegalLinks, Network, PublicUrl};

    pub const PUBLIC_URL: &str = "https://adunata.example.org";

    pub fn state(pool: PgPool) -> AppState {
        AppState {
            pool,
            public_url: PublicUrl::parse(PUBLIC_URL).expect("test URL must be valid"),
            default_language: "de".to_owned(),
            legal_links: LegalLinks::default(),
            trusted_proxies: vec![Network::parse("127.0.0.1/32").expect("loopback is a network")],
            sign_in_limiter: SignInLimiter::default(),
            time_zone: InstanceTimeZone::parse("Europe/Berlin")
                .expect("Berlin is a known time zone"),
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Method, Request, StatusCode, header};
    use tower::ServiceExt;
    use utoipa_axum::routes;

    use super::*;

    /// Operations that answer without an organizer session: health, catalogs, the links every page shows, sign-in and sign-out, the routes behind an invitation link and the spike until feature modules replace it.
    const PUBLIC: [&str; 7] = [
        "GET /api/health",
        "GET /api/catalogs/{locale}",
        "GET /api/legal-links",
        "POST /api/sign-in-requests",
        "POST /api/sessions",
        "DELETE /api/session",
        "POST /api/spike-responses",
    ];

    /// Replaces each path parameter such as `{id}` with `1`.
    fn with_example_parameters(path: &str) -> String {
        path.split('/')
            .map(|segment| {
                if segment.starts_with('{') {
                    "1"
                } else {
                    segment
                }
            })
            .collect::<Vec<_>>()
            .join("/")
    }

    /// Calls every operation of the document without a session and returns those that do not answer with status 401 and are not public.
    async fn unprotected(operations: OpenApiRouter<AppState>) -> Vec<String> {
        let (router, document) = operations.split_for_parts();
        let pool = sqlx::PgPool::connect_lazy("postgres://localhost").unwrap();
        let app = router.with_state(testing::state(pool));
        let mut found = Vec::new();
        for (path, item) in &document.paths.paths {
            let verbs = [
                (Method::GET, &item.get),
                (Method::PUT, &item.put),
                (Method::POST, &item.post),
                (Method::DELETE, &item.delete),
                (Method::PATCH, &item.patch),
            ];
            for (verb, _) in verbs
                .into_iter()
                .filter(|(_, operation)| operation.is_some())
            {
                let name = format!("{verb} {path}");
                if PUBLIC.contains(&name.as_str()) {
                    continue;
                }
                let uri = with_example_parameters(path);
                let request = Request::builder()
                    .method(verb)
                    .uri(uri)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from("{}"))
                    .unwrap();
                let status = app.clone().oneshot(request).await.unwrap().status();
                if status != StatusCode::UNAUTHORIZED {
                    found.push(format!("{name} answered {status}"));
                }
            }
        }
        found
    }

    #[tokio::test]
    async fn every_organizer_operation_requires_a_session() {
        assert_eq!(unprotected(routes()).await, Vec::<String>::new());
    }

    #[utoipa::path(post, path = "/api/forgotten", responses((status = 204, description = "Changes data without asking who")))]
    async fn forgotten() -> StatusCode {
        StatusCode::NO_CONTENT
    }

    #[tokio::test]
    async fn a_route_without_the_extractor_is_found() {
        let routes = routes().merge(OpenApiRouter::new().routes(routes!(forgotten)));
        assert_eq!(
            unprotected(routes).await,
            vec!["POST /api/forgotten answered 204 No Content".to_owned()]
        );
    }
}
