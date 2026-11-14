use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use time::Date;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use fluent_bundle::FluentArgs;

use crate::api::AppState;
use crate::configuration::PublicUrl;
use crate::localization;
use crate::mail::{self, Mail};

const MAX_NAME_LENGTH: usize = 100;

time::serde::format_description!(iso_date, Date, "[year]-[month]-[day]");
const MAX_BEDS: u8 = 20;
const SPIKE_RECIPIENT: &str = "Organizer <organizer@example.org>";
const ORGANIZER_LOCALE: &str = "de";
const GATHERING_START: Date = time::macros::date!(2027 - 03 - 14);
const GATHERING_END: Date = time::macros::date!(2027 - 03 - 16);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum Attendance {
    Accepted,
    Tentative,
    Declined,
}

impl Attendance {
    fn as_str(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::Tentative => "tentative",
            Self::Declined => "declined",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SpikeResponse {
    #[schema(min_length = 1, max_length = 100)]
    pub name: String,
    pub attendance: Attendance,
    #[serde(default, with = "iso_date::option")]
    pub arrival: Option<Date>,
    #[schema(minimum = 0, maximum = 20)]
    pub beds: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum ErrorCode {
    Required,
    TooLong,
    TooLarge,
    Taken,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct FieldError {
    pub field: String,
    pub code: ErrorCode,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ValidationErrors {
    pub errors: Vec<FieldError>,
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(create_response))
}

#[utoipa::path(
    post,
    path = "/api/spike-responses",
    request_body = SpikeResponse,
    responses(
        (status = 201, description = "Response stored", body = SpikeResponse),
        (status = 422, description = "Response invalid", body = ValidationErrors),
    )
)]
async fn create_response(
    State(state): State<AppState>,
    Json(response): Json<SpikeResponse>,
) -> Response {
    let errors = validate(&response);
    if !errors.is_empty() {
        return invalid(errors);
    }
    match insert(&state, &response).await {
        Ok(()) => (StatusCode::CREATED, Json(response)).into_response(),
        Err(error) if is_unique_violation(&error) => {
            invalid(vec![field_error("name", ErrorCode::Taken)])
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

fn validate(response: &SpikeResponse) -> Vec<FieldError> {
    let mut errors = Vec::new();
    let name_length = response.name.trim().chars().count();
    if name_length == 0 {
        errors.push(field_error("name", ErrorCode::Required));
    } else if name_length > MAX_NAME_LENGTH {
        errors.push(field_error("name", ErrorCode::TooLong));
    }
    if response.beds > MAX_BEDS {
        errors.push(field_error("beds", ErrorCode::TooLarge));
    }
    errors
}

async fn insert(state: &AppState, response: &SpikeResponse) -> Result<(), sqlx::Error> {
    let mut transaction = state.pool.begin().await?;
    sqlx::query(
        "INSERT INTO spike_responses (name, attendance, arrival, beds) VALUES ($1, $2, $3, $4)",
    )
    .bind(response.name.trim())
    .bind(response.attendance.as_str())
    .bind(response.arrival)
    .bind(i16::from(response.beds))
    .execute(&mut *transaction)
    .await?;
    mail::enqueue(&mut transaction, &confirmation(response, &state.public_url)).await?;
    transaction.commit().await
}

fn confirmation(response: &SpikeResponse, public_url: &PublicUrl) -> Mail {
    let localizer = localization::localizer();
    let name = response.name.trim();
    let mut args = FluentArgs::new();
    args.set("name", name);
    args.set("attendance", response.attendance.as_str());
    args.set(
        "span",
        localizer.date_span(ORGANIZER_LOCALE, GATHERING_START, GATHERING_END),
    );
    args.set("beds", response.beds);
    let subject = localizer.message(ORGANIZER_LOCALE, "response-mail-subject", &args);
    let body = localizer.message(ORGANIZER_LOCALE, "response-mail-body", &args);
    let beds = localizer.message(ORGANIZER_LOCALE, "beds-needed", &args);
    let link = public_url.link("/");
    Mail {
        recipient: SPIKE_RECIPIENT.to_owned(),
        subject,
        text: format!("{body}\n{beds}\n{link}"),
        html: format!(
            "<p>{}</p><p>{}</p><p><a href=\"{link}\">{link}</a></p>",
            escape_html(&body),
            escape_html(&beds)
        ),
    }
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn is_unique_violation(error: &sqlx::Error) -> bool {
    error
        .as_database_error()
        .is_some_and(sqlx::error::DatabaseError::is_unique_violation)
}

fn field_error(field: &str, code: ErrorCode) -> FieldError {
    FieldError {
        field: field.to_owned(),
        code,
    }
}

fn invalid(errors: Vec<FieldError>) -> Response {
    (
        StatusCode::UNPROCESSABLE_ENTITY,
        Json(ValidationErrors { errors }),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use crate::configuration::PublicUrl;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, header};
    use serde_json::{Value, json};
    use tower::ServiceExt;

    use crate::api;
    use crate::database::testing::fresh_database;

    async fn post(app: axum::Router, body: &Value) -> (axum::http::StatusCode, Value) {
        let request = Request::post("/api/spike-responses")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();
        let response = app.oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        (
            status,
            serde_json::from_slice(&bytes).unwrap_or(Value::Null),
        )
    }

    fn ada() -> Value {
        json!({ "name": "Ada", "attendance": "accepted", "arrival": "2027-03-14", "beds": 2 })
    }

    #[tokio::test]
    async fn stores_a_valid_response() {
        let pool = fresh_database().await;
        let (status, _) = post(
            api::router(crate::api::testing::state(pool.clone())),
            &ada(),
        )
        .await;
        assert_eq!(status, 201);
        let counts: (i64, i64) = sqlx::query_as(
            "SELECT (SELECT count(*) FROM spike_responses), (SELECT count(*) FROM mail_outbox)",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(counts, (1, 1));
    }

    #[tokio::test]
    async fn rejects_a_taken_name_at_the_name_field() {
        let pool = fresh_database().await;
        post(
            api::router(crate::api::testing::state(pool.clone())),
            &ada(),
        )
        .await;
        let (status, body) = post(
            api::router(crate::api::testing::state(pool.clone())),
            &ada(),
        )
        .await;
        assert_eq!(status, 422);
        assert_eq!(
            body,
            json!({ "errors": [{ "field": "name", "code": "taken" }] })
        );
        let mails: i64 = sqlx::query_scalar("SELECT count(*) FROM mail_outbox")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(mails, 1, "a rejected response must not leave a mail behind");
    }

    #[test]
    fn confirmation_is_localized_and_escaped() {
        let response = super::SpikeResponse {
            name: "Ada <b>".to_owned(),
            attendance: super::Attendance::Tentative,
            arrival: None,
            beds: 2,
        };
        let public_url = PublicUrl::parse(crate::api::testing::PUBLIC_URL).unwrap();
        let mail = super::confirmation(&response, &public_url);
        assert_eq!(mail.subject, "Rückmeldung von Ada <b>");
        assert_eq!(
            mail.text,
            "Ada <b> hat die Einladung zur Zusammenkunft vom 14.–16. März 2027 vorläufig zugesagt.\n2 Betten benötigt.\nhttps://adunata.example.org/"
        );
        assert!(mail.html.contains("Ada &lt;b&gt;"));
    }

    #[test]
    fn link_in_a_mail() {
        let response = super::SpikeResponse {
            name: "Ada".to_owned(),
            attendance: super::Attendance::Accepted,
            arrival: None,
            beds: 0,
        };
        let public_url = PublicUrl::parse("https://choir.example.org/adunata/").unwrap();
        let mail = super::confirmation(&response, &public_url);
        assert!(mail.text.contains("https://choir.example.org/adunata/"));
        assert!(
            mail.html
                .contains("href=\"https://choir.example.org/adunata/\"")
        );
    }

    #[tokio::test]
    async fn rejects_a_blank_name() {
        let pool = fresh_database().await;
        let (status, body) = post(
            api::router(crate::api::testing::state(pool)),
            &json!({ "name": "  ", "attendance": "declined", "beds": 0 }),
        )
        .await;
        assert_eq!(status, 422);
        assert_eq!(
            body,
            json!({ "errors": [{ "field": "name", "code": "required" }] })
        );
    }
}
