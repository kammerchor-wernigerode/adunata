use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use sqlx::PgConnection;
use time::Date;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::access::Organizer;
use crate::api::AppState;
use crate::validation::{ErrorCode, FieldError, field_error, invalid};

time::serde::format_description!(iso_date, Date, "[year]-[month]-[day]");

const MAX_TITLE_LENGTH: usize = 200;
const MAX_LOCATION_LENGTH: usize = 200;
const MAX_DESCRIPTION_LENGTH: usize = 5000;
const COLUMNS: &str =
    "id, title, location, description, first_day, last_day, cancelled_at IS NOT NULL AS cancelled";

#[derive(sqlx::FromRow)]
struct Row {
    id: i64,
    title: String,
    location: String,
    description: Option<String>,
    first_day: Date,
    last_day: Date,
    cancelled: bool,
}

/// A span of days on which the ensemble meets, as organizers see it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Gathering {
    pub id: i64,
    pub title: String,
    pub location: String,
    pub description: Option<String>,
    #[serde(with = "iso_date")]
    #[schema(value_type = String, format = Date)]
    pub first_day: Date,
    #[serde(with = "iso_date")]
    #[schema(value_type = String, format = Date)]
    pub last_day: Date,
    pub cancelled: bool,
    /// Whether the last day has passed in the time zone of the instance.
    pub ended: bool,
}

impl Gathering {
    fn from_row(row: Row, today: Date) -> Self {
        Self {
            id: row.id,
            title: row.title,
            location: row.location,
            description: row.description,
            first_day: row.first_day,
            last_day: row.last_day,
            cancelled: row.cancelled,
            ended: row.last_day < today,
        }
    }

    /// Whether the first day has begun in the time zone of the instance.
    pub fn has_begun(&self, today: Date) -> bool {
        self.first_day <= today
    }
}

/// What an organizer enters for a gathering.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GatheringInput {
    pub title: String,
    pub location: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(with = "iso_date")]
    #[schema(value_type = String, format = Date)]
    pub first_day: Date,
    #[serde(with = "iso_date")]
    #[schema(value_type = String, format = Date)]
    pub last_day: Date,
}

/// Gatherings split at the present. Upcoming ones come first, ordered by first day. Past ones follow with the latest first.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct GatheringList {
    pub upcoming: Vec<Gathering>,
    pub past: Vec<Gathering>,
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list_gatherings, create_gathering))
        .routes(routes!(get_gathering, change_gathering))
}

#[utoipa::path(
    get,
    path = "/api/gatherings",
    responses(
        (status = 200, description = "Upcoming and past gatherings", body = GatheringList),
        (status = 401, description = "No valid organizer session"),
    )
)]
async fn list_gatherings(_: Organizer, State(state): State<AppState>) -> Response {
    let query = format!("SELECT {COLUMNS} FROM gatherings ORDER BY first_day, id");
    let Ok(rows) = sqlx::query_as::<_, Row>(sqlx::AssertSqlSafe(query))
        .fetch_all(&state.pool)
        .await
    else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    let today = state.time_zone.today();
    let (past, upcoming): (Vec<Gathering>, Vec<Gathering>) = rows
        .into_iter()
        .map(|row| Gathering::from_row(row, today))
        .partition(|gathering| gathering.ended);
    Json(GatheringList {
        upcoming,
        past: past.into_iter().rev().collect(),
    })
    .into_response()
}

#[utoipa::path(
    post,
    path = "/api/gatherings",
    request_body = GatheringInput,
    responses(
        (status = 201, description = "Gathering created", body = Gathering),
        (status = 401, description = "No valid organizer session"),
        (status = 422, description = "Gathering invalid", body = crate::validation::ValidationErrors),
    )
)]
async fn create_gathering(
    _: Organizer,
    State(state): State<AppState>,
    Json(input): Json<GatheringInput>,
) -> Response {
    let errors = validate(&input);
    if !errors.is_empty() {
        return invalid(errors);
    }
    let query = format!(
        "INSERT INTO gatherings (title, location, description, first_day, last_day) VALUES ($1, $2, $3, $4, $5) RETURNING {COLUMNS}"
    );
    match bind(sqlx::query_as::<_, Row>(sqlx::AssertSqlSafe(query)), &input)
        .fetch_one(&state.pool)
        .await
    {
        Ok(row) => (
            StatusCode::CREATED,
            Json(Gathering::from_row(row, state.time_zone.today())),
        )
            .into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/api/gatherings/{id}",
    params(("id" = i64, Path, description = "Id of the gathering")),
    responses(
        (status = 200, description = "The gathering", body = Gathering),
        (status = 401, description = "No valid organizer session"),
        (status = 404, description = "No such gathering"),
    )
)]
async fn get_gathering(
    _: Organizer,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Response {
    let Ok(mut connection) = state.pool.acquire().await else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    match find(&mut connection, id, state.time_zone.today()).await {
        Ok(Some(gathering)) => Json(gathering).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[utoipa::path(
    put,
    path = "/api/gatherings/{id}",
    params(("id" = i64, Path, description = "Id of the gathering")),
    request_body = GatheringInput,
    responses(
        (status = 200, description = "Gathering changed", body = Gathering),
        (status = 401, description = "No valid organizer session"),
        (status = 404, description = "No such gathering"),
        (status = 422, description = "Change invalid or the gathering has ended", body = crate::validation::ValidationErrors),
    )
)]
async fn change_gathering(
    _: Organizer,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(input): Json<GatheringInput>,
) -> Response {
    let errors = validate(&input);
    if !errors.is_empty() {
        return invalid(errors);
    }
    let query = format!(
        "UPDATE gatherings SET title = $1, location = $2, description = $3, first_day = $4, last_day = $5 WHERE id = $6 AND last_day >= $7 RETURNING {COLUMNS}"
    );
    let today = state.time_zone.today();
    let updated = bind(sqlx::query_as::<_, Row>(sqlx::AssertSqlSafe(query)), &input)
        .bind(id)
        .bind(today)
        .fetch_optional(&state.pool)
        .await;
    match updated {
        Ok(Some(row)) => Json(Gathering::from_row(row, today)).into_response(),
        Ok(None) => match state.pool.acquire().await {
            Ok(mut connection) => match find(&mut connection, id, today).await {
                Ok(Some(_)) => invalid(vec![field_error("gathering", ErrorCode::Ended)]),
                Ok(None) => StatusCode::NOT_FOUND.into_response(),
                Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            },
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        },
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// Marks a gathering as cancelled unless it is cancelled already or has ended. Returns the gathering as it was cancelled, or `None` when nothing changed.
pub async fn cancel(
    connection: &mut PgConnection,
    id: i64,
    today: Date,
) -> Result<Option<Gathering>, sqlx::Error> {
    let query = format!(
        "UPDATE gatherings SET cancelled_at = now() WHERE id = $1 AND cancelled_at IS NULL AND last_day >= $2 RETURNING {COLUMNS}"
    );
    Ok(sqlx::query_as::<_, Row>(sqlx::AssertSqlSafe(query))
        .bind(id)
        .bind(today)
        .fetch_optional(connection)
        .await?
        .map(|row| Gathering::from_row(row, today)))
}

/// The gathering with this id, judged against the given date.
pub async fn find(
    connection: &mut PgConnection,
    id: i64,
    today: Date,
) -> Result<Option<Gathering>, sqlx::Error> {
    let query = format!("SELECT {COLUMNS} FROM gatherings WHERE id = $1");
    Ok(sqlx::query_as::<_, Row>(sqlx::AssertSqlSafe(query))
        .bind(id)
        .fetch_optional(connection)
        .await?
        .map(|row| Gathering::from_row(row, today)))
}

fn bind<'q>(
    query: sqlx::query::QueryAs<'q, sqlx::Postgres, Row, sqlx::postgres::PgArguments>,
    input: &'q GatheringInput,
) -> sqlx::query::QueryAs<'q, sqlx::Postgres, Row, sqlx::postgres::PgArguments> {
    query
        .bind(input.title.trim())
        .bind(input.location.trim())
        .bind(
            input
                .description
                .as_deref()
                .map(str::trim)
                .filter(|description| !description.is_empty()),
        )
        .bind(input.first_day)
        .bind(input.last_day)
}

fn validate(input: &GatheringInput) -> Vec<FieldError> {
    let mut errors = Vec::new();
    for (field, value, limit) in [
        ("title", &input.title, MAX_TITLE_LENGTH),
        ("location", &input.location, MAX_LOCATION_LENGTH),
    ] {
        let length = value.trim().chars().count();
        if length == 0 {
            errors.push(field_error(field, ErrorCode::Required));
        } else if length > limit {
            errors.push(field_error(field, ErrorCode::TooLong));
        }
    }
    if input
        .description
        .as_deref()
        .is_some_and(|description| description.trim().chars().count() > MAX_DESCRIPTION_LENGTH)
    {
        errors.push(field_error("description", ErrorCode::TooLong));
    }
    if input.last_day < input.first_day {
        errors.push(field_error("lastDay", ErrorCode::BeforeFirstDay));
    }
    errors
}

#[cfg(test)]
mod tests {
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, header};
    use serde_json::{Value, json};
    use time::Duration;
    use tower::ServiceExt;

    use super::*;
    use crate::access::testing::signed_in_organizer;
    use crate::api;
    use crate::database::testing::fresh_database;

    async fn send(
        pool: &sqlx::PgPool,
        cookie: &str,
        method: &str,
        path: &str,
        body: &Value,
    ) -> (StatusCode, Value) {
        let request = Request::builder()
            .method(method)
            .uri(path)
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::COOKIE, cookie)
            .body(Body::from(body.to_string()))
            .unwrap();
        let response = api::router(api::testing::state(pool.clone()))
            .oneshot(request)
            .await
            .unwrap();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        (
            status,
            serde_json::from_slice(&bytes).unwrap_or(Value::Null),
        )
    }

    fn today() -> Date {
        crate::configuration::InstanceTimeZone::parse("Europe/Berlin")
            .unwrap()
            .today()
    }

    fn day(offset: i64) -> String {
        (today() + Duration::days(offset)).to_string()
    }

    fn weekend(first: i64, last: i64) -> Value {
        json!({ "title": "Probenwochenende", "location": "Kloster Drübeck", "firstDay": day(first), "lastDay": day(last) })
    }

    #[tokio::test]
    async fn create_a_gathering() {
        let database = fresh_database().await;
        let pool = database.pool();
        let (_, cookie) = signed_in_organizer(&pool).await;
        let (status, created) =
            send(&pool, &cookie, "POST", "/api/gatherings", &weekend(10, 12)).await;
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(created["cancelled"], false);
        assert_eq!(created["ended"], false);
        assert_eq!(created["description"], Value::Null);
        let (_, list) = send(&pool, &cookie, "GET", "/api/gatherings", &Value::Null).await;
        assert_eq!(list["upcoming"][0]["id"], created["id"]);
        assert_eq!(list["past"], json!([]));
    }

    #[tokio::test]
    async fn last_day_before_first_day() {
        let database = fresh_database().await;
        let pool = database.pool();
        let (_, cookie) = signed_in_organizer(&pool).await;
        let (status, body) =
            send(&pool, &cookie, "POST", "/api/gatherings", &weekend(12, 10)).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            body,
            json!({ "errors": [{ "field": "lastDay", "code": "beforeFirstDay" }] })
        );
    }

    #[tokio::test]
    async fn gathering_has_ended() {
        let database = fresh_database().await;
        let pool = database.pool();
        let (_, cookie) = signed_in_organizer(&pool).await;
        for (first, last) in [(-3, -1), (-10, -8), (0, 0), (5, 6), (2, 3)] {
            send(
                &pool,
                &cookie,
                "POST",
                "/api/gatherings",
                &weekend(first, last),
            )
            .await;
        }
        let (_, list) = send(&pool, &cookie, "GET", "/api/gatherings", &Value::Null).await;
        let firsts = |key: &str| -> Vec<String> {
            list[key]
                .as_array()
                .unwrap()
                .iter()
                .map(|gathering| gathering["firstDay"].as_str().unwrap().to_owned())
                .collect()
        };
        assert_eq!(firsts("upcoming"), vec![day(0), day(2), day(5)]);
        assert_eq!(firsts("past"), vec![day(-3), day(-10)]);
        assert!(
            list["past"]
                .as_array()
                .unwrap()
                .iter()
                .all(|gathering| gathering["ended"] == true)
        );
    }

    #[tokio::test]
    async fn edit_a_gathering_that_has_not_ended() {
        let database = fresh_database().await;
        let pool = database.pool();
        let (_, cookie) = signed_in_organizer(&pool).await;
        let (_, created) = send(&pool, &cookie, "POST", "/api/gatherings", &weekend(0, 2)).await;
        let mut change = weekend(1, 2);
        change["description"] = json!("Wir proben das Programm für April.");
        let (status, changed) = send(
            &pool,
            &cookie,
            "PUT",
            &format!("/api/gatherings/{}", created["id"]),
            &change,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(changed["firstDay"], day(1));
        assert_eq!(changed["description"], "Wir proben das Programm für April.");
    }

    #[tokio::test]
    async fn ended_gathering_stays_as_it_was() {
        let database = fresh_database().await;
        let pool = database.pool();
        let (_, cookie) = signed_in_organizer(&pool).await;
        let (_, created) = send(&pool, &cookie, "POST", "/api/gatherings", &weekend(-3, -1)).await;
        let (status, body) = send(
            &pool,
            &cookie,
            "PUT",
            &format!("/api/gatherings/{}", created["id"]),
            &weekend(-3, 0),
        )
        .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            body,
            json!({ "errors": [{ "field": "gathering", "code": "ended" }] })
        );
        let (status, _) = send(&pool, &cookie, "PUT", "/api/gatherings/999", &weekend(1, 2)).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[test]
    fn begins_on_its_first_day() {
        let gathering = Gathering::from_row(
            Row {
                id: 1,
                title: "t".to_owned(),
                location: "l".to_owned(),
                description: None,
                first_day: today(),
                last_day: today(),
                cancelled: false,
            },
            today(),
        );
        assert!(gathering.has_begun(today()));
        assert!(!gathering.has_begun(today() - Duration::days(1)));
        assert!(!gathering.ended);
    }
}
