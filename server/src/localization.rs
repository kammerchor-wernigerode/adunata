use std::collections::HashMap;
use std::sync::LazyLock;

use axum::extract::Path;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentArgs, FluentResource};
use sqlx::PgPool;
use time::Date;
use unic_langid::LanguageIdentifier;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

const FALLBACK: &str = "en";
const CATALOGS: [(&str, &str); 2] = [
    ("en", include_str!("../locales/en.ftl")),
    ("de", include_str!("../locales/de.ftl")),
];

static LOCALIZER: LazyLock<Localizer> = LazyLock::new(Localizer::new);

pub fn localizer() -> &'static Localizer {
    &LOCALIZER
}

pub struct Localizer {
    bundles: HashMap<&'static str, FluentBundle<FluentResource>>,
}

impl Localizer {
    pub fn new() -> Self {
        let bundles = CATALOGS
            .into_iter()
            .map(|(locale, source)| (locale, bundle(locale, source)))
            .collect();
        Self { bundles }
    }

    pub fn message(&self, locale: &str, id: &str, args: &FluentArgs) -> String {
        [locale, FALLBACK]
            .into_iter()
            .filter_map(|candidate| self.bundles.get(candidate))
            .find_map(|bundle| format(bundle, id, args))
            .unwrap_or_else(|| id.to_owned())
    }

    pub fn date_span(&self, locale: &str, start: Date, end: Date) -> String {
        let mut args = FluentArgs::new();
        args.set("startDay", start.day());
        args.set("endDay", end.day());
        args.set("year", start.year());
        let month_id = format!("month-{}", u8::from(start.month()));
        args.set("month", self.message(locale, &month_id, &FluentArgs::new()));
        self.message(locale, "date-span-same-month", &args)
    }
}

fn bundle(locale: &str, source: &str) -> FluentBundle<FluentResource> {
    let language: LanguageIdentifier = locale.parse().expect("catalog locale must be valid");
    let resource = FluentResource::try_new(source.to_owned()).expect("catalog must parse");
    let mut bundle = FluentBundle::new_concurrent(vec![language]);
    bundle.set_use_isolating(false);
    bundle
        .add_resource(resource)
        .expect("catalog must not define a message twice");
    bundle
}

fn format(bundle: &FluentBundle<FluentResource>, id: &str, args: &FluentArgs) -> Option<String> {
    let pattern = bundle.get_message(id)?.value()?;
    let mut errors = Vec::new();
    Some(
        bundle
            .format_pattern(pattern, Some(args), &mut errors)
            .into_owned(),
    )
}

pub fn router() -> OpenApiRouter<PgPool> {
    OpenApiRouter::new().routes(routes!(catalog))
}

#[utoipa::path(
    get,
    path = "/api/catalogs/{locale}",
    params(("locale" = String, Path, description = "Language of the catalog, such as de or en")),
    responses(
        (status = 200, description = "Fluent catalog", content_type = "text/plain", body = String),
        (status = 404, description = "No catalog for this language"),
    )
)]
async fn catalog(Path(locale): Path<String>) -> Response {
    CATALOGS
        .iter()
        .find(|(candidate, _)| *candidate == locale)
        .map_or_else(
            || StatusCode::NOT_FOUND.into_response(),
            |(_, source)| {
                (
                    [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
                    *source,
                )
                    .into_response()
            },
        )
}

#[cfg(test)]
mod tests {
    use time::macros::date;

    use super::*;

    fn beds(localizer: &Localizer, locale: &str, count: u8) -> String {
        let mut args = FluentArgs::new();
        args.set("beds", count);
        localizer.message(locale, "beds-needed", &args)
    }

    #[test]
    fn plural_forms_in_english() {
        let localizer = Localizer::new();
        assert_eq!(beds(&localizer, "en", 0), "0 beds needed.");
        assert_eq!(beds(&localizer, "en", 1), "One bed needed.");
        assert_eq!(beds(&localizer, "en", 2), "2 beds needed.");
    }

    #[test]
    fn plural_forms_in_german() {
        let localizer = Localizer::new();
        assert_eq!(beds(&localizer, "de", 0), "0 Betten benötigt.");
        assert_eq!(beds(&localizer, "de", 1), "Ein Bett benötigt.");
        assert_eq!(beds(&localizer, "de", 2), "2 Betten benötigt.");
    }

    #[test]
    fn date_span_per_language() {
        let localizer = Localizer::new();
        let (start, end) = (date!(2027 - 03 - 14), date!(2027 - 03 - 16));
        assert_eq!(localizer.date_span("de", start, end), "14.–16. März 2027");
        assert_eq!(localizer.date_span("en", start, end), "March 14–16, 2027");
    }

    #[tokio::test]
    async fn serves_known_catalogs_only() {
        use axum::body::{Body, to_bytes};
        use axum::http::Request;
        use tower::ServiceExt;

        let pool = PgPool::connect_lazy("postgres://localhost").unwrap();
        let app: axum::Router = router().split_for_parts().0.with_state(pool);
        let german = app
            .clone()
            .oneshot(
                Request::get("/api/catalogs/de")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(german.status(), StatusCode::OK);
        let body = to_bytes(german.into_body(), usize::MAX).await.unwrap();
        assert!(String::from_utf8_lossy(&body).contains("Betten benötigt"));
        let french = app
            .oneshot(
                Request::get("/api/catalogs/fr")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(french.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn unknown_language_falls_back_to_english() {
        assert_eq!(beds(&Localizer::new(), "fr", 2), "2 beds needed.");
    }
}
