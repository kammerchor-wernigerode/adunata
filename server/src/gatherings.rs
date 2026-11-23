use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use sqlx::PgConnection;
use time::{Date, OffsetDateTime};
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

/// How often a gathering changed since its creation and when it changed last.
#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct Revision {
    pub created_at: OffsetDateTime,
    pub changed_at: OffsetDateTime,
    /// The number of stored changes and cancellations.
    pub version: i32,
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
    let errors = validate(&input, state.time_zone.today(), None);
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
    change(&state, id, &input)
        .await
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

/// Changes a gathering under the lock of its row, so the stored first day cannot change between validation and update.
async fn change(
    state: &AppState,
    id: i64,
    input: &GatheringInput,
) -> Result<Response, sqlx::Error> {
    let today = state.time_zone.today();
    let mut transaction = state.pool.begin().await?;
    let Some(stored) = find_for_update(&mut transaction, id, today).await? else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };
    if stored.ended {
        return Ok(invalid(vec![field_error("gathering", ErrorCode::Ended)]));
    }
    let errors = validate(input, today, Some(stored.first_day));
    if !errors.is_empty() {
        return Ok(invalid(errors));
    }
    let query = format!(
        "UPDATE gatherings SET title = $1, location = $2, description = $3, first_day = $4, last_day = $5, version = version + 1, changed_at = now() WHERE id = $6 RETURNING {COLUMNS}"
    );
    let row = bind(sqlx::query_as::<_, Row>(sqlx::AssertSqlSafe(query)), input)
        .bind(id)
        .fetch_one(&mut *transaction)
        .await?;
    transaction.commit().await?;
    Ok(Json(Gathering::from_row(row, today)).into_response())
}

/// Marks a gathering as cancelled unless it is cancelled already or has ended. Returns the gathering as it was cancelled, or `None` when nothing changed.
pub async fn cancel(
    connection: &mut PgConnection,
    id: i64,
    today: Date,
) -> Result<Option<Gathering>, sqlx::Error> {
    let query = format!(
        "UPDATE gatherings SET cancelled_at = now(), version = version + 1, changed_at = now() WHERE id = $1 AND cancelled_at IS NULL AND last_day >= $2 RETURNING {COLUMNS}"
    );
    Ok(sqlx::query_as::<_, Row>(sqlx::AssertSqlSafe(query))
        .bind(id)
        .bind(today)
        .fetch_optional(connection)
        .await?
        .map(|row| Gathering::from_row(row, today)))
}

/// The gathering with this id, judged against the given date. Locks its row until the transaction ends, so a cancellation and an invitation of the same gathering cannot interleave.
pub async fn find_for_update(
    connection: &mut PgConnection,
    id: i64,
    today: Date,
) -> Result<Option<Gathering>, sqlx::Error> {
    let query = format!("SELECT {COLUMNS} FROM gatherings WHERE id = $1 FOR UPDATE");
    Ok(sqlx::query_as::<_, Row>(sqlx::AssertSqlSafe(query))
        .bind(id)
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

/// The revision of the gathering with this id.
pub async fn revision(connection: &mut PgConnection, id: i64) -> Result<Revision, sqlx::Error> {
    sqlx::query_as("SELECT created_at, changed_at, version FROM gatherings WHERE id = $1")
        .bind(id)
        .fetch_one(connection)
        .await
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

/// The errors of each field. A first day before today is refused unless it is the stored first day of the gathering that changes.
fn validate(
    input: &GatheringInput,
    today: Date,
    stored_first_day: Option<Date>,
) -> Vec<FieldError> {
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
    if input.first_day < today && Some(input.first_day) != stored_first_day {
        errors.push(field_error("firstDay", ErrorCode::FirstDayInPast));
    }
    if input.last_day < input.first_day {
        errors.push(field_error("lastDay", ErrorCode::BeforeFirstDay));
    }
    errors
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::Duration;
    use time::macros::date;

    fn fixed_day() -> Date {
        date!(2027 - 03 - 14)
    }

    #[test]
    fn begins_on_its_first_day() {
        let gathering = Gathering::from_row(
            Row {
                id: 1,
                title: "t".to_owned(),
                location: "l".to_owned(),
                description: None,
                first_day: fixed_day(),
                last_day: fixed_day(),
                cancelled: false,
            },
            fixed_day(),
        );
        assert!(gathering.has_begun(fixed_day()));
        assert!(!gathering.has_begun(fixed_day() - Duration::days(1)));
        assert!(!gathering.ended);
    }

    fn input(first_day: Date, last_day: Date) -> GatheringInput {
        GatheringInput {
            title: "Probenwochenende".to_owned(),
            location: "Kloster Drübeck".to_owned(),
            description: None,
            first_day,
            last_day,
        }
    }

    fn first_day_in_past() -> Vec<FieldError> {
        vec![field_error("firstDay", ErrorCode::FirstDayInPast)]
    }

    #[test]
    fn refuses_new_first_day_in_past() {
        let yesterday = fixed_day() - Duration::days(1);
        assert_eq!(
            validate(&input(yesterday, fixed_day()), fixed_day(), None),
            first_day_in_past()
        );
    }

    #[test]
    fn accepts_today_as_first_and_last_day() {
        assert_eq!(
            validate(&input(fixed_day(), fixed_day()), fixed_day(), None),
            vec![]
        );
    }

    #[test]
    fn accepts_unchanged_first_day_in_past() {
        let stored = fixed_day() - Duration::days(2);
        assert_eq!(
            validate(&input(stored, fixed_day()), fixed_day(), Some(stored)),
            vec![]
        );
    }

    #[test]
    fn refuses_changed_first_day_in_past() {
        let stored = fixed_day() - Duration::days(2);
        let changed = fixed_day() - Duration::days(1);
        assert_eq!(
            validate(&input(changed, fixed_day()), fixed_day(), Some(stored)),
            first_day_in_past()
        );
    }

    mod integration {
        use super::*;
        use crate::access::testing::signed_in_organizer;
        use crate::api;
        use crate::database::testing::fresh_database;
        use axum::body::{Body, to_bytes};
        use axum::http::{Request, header};
        use serde_json::{Value, json};
        use tower::ServiceExt;

        fn today() -> Date {
            api::testing::day(0)
        }

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
        async fn new_gathering_starts_in_the_past() {
            let database = fresh_database().await;
            let pool = database.pool();
            let (_, cookie) = signed_in_organizer(&pool).await;
            let (status, body) =
                send(&pool, &cookie, "POST", "/api/gatherings", &weekend(-1, 1)).await;
            assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
            assert_eq!(
                body,
                json!({ "errors": [{ "field": "firstDay", "code": "firstDayInPast" }] })
            );
            let (status, _) = send(&pool, &cookie, "POST", "/api/gatherings", &weekend(0, 0)).await;
            assert_eq!(status, StatusCode::CREATED);
        }

        #[tokio::test]
        async fn gathering_has_ended() {
            let database = fresh_database().await;
            let pool = database.pool();
            let (_, cookie) = signed_in_organizer(&pool).await;
            for (first, last) in [(-3, -1), (-10, -8), (0, 0), (5, 6), (2, 3)] {
                crate::invitations::testing::gathering(&pool, "Probenwochenende", first, last)
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
            let (_, created) =
                send(&pool, &cookie, "POST", "/api/gatherings", &weekend(0, 2)).await;
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
            let id =
                crate::invitations::testing::gathering(&pool, "Probenwochenende", -3, -1).await;
            let (status, body) = send(
                &pool,
                &cookie,
                "PUT",
                &format!("/api/gatherings/{id}"),
                &weekend(-3, 0),
            )
            .await;
            assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
            assert_eq!(
                body,
                json!({ "errors": [{ "field": "gathering", "code": "ended" }] })
            );
            let (status, _) =
                send(&pool, &cookie, "PUT", "/api/gatherings/999", &weekend(1, 2)).await;
            assert_eq!(status, StatusCode::NOT_FOUND);
        }

        #[tokio::test]
        async fn unknown_gathering_with_invalid_change() {
            let database = fresh_database().await;
            let pool = database.pool();
            let (_, cookie) = signed_in_organizer(&pool).await;
            let (status, _) =
                send(&pool, &cookie, "PUT", "/api/gatherings/999", &weekend(2, 1)).await;
            assert_eq!(status, StatusCode::NOT_FOUND);
        }

        #[tokio::test]
        async fn running_gathering_keeps_its_first_day() {
            let database = fresh_database().await;
            let pool = database.pool();
            let (_, cookie) = signed_in_organizer(&pool).await;
            let id = crate::invitations::testing::gathering(&pool, "Probenwochenende", -2, 2).await;
            let path = format!("/api/gatherings/{id}");
            let (status, changed) = send(&pool, &cookie, "PUT", &path, &weekend(-2, 3)).await;
            assert_eq!(status, StatusCode::OK);
            assert_eq!(changed["lastDay"], day(3));
            let (status, body) = send(&pool, &cookie, "PUT", &path, &weekend(-1, 3)).await;
            assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
            assert_eq!(
                body,
                json!({ "errors": [{ "field": "firstDay", "code": "firstDayInPast" }] })
            );
            let (_, stored) = send(&pool, &cookie, "GET", &path, &Value::Null).await;
            assert_eq!(stored["firstDay"], day(-2));
            assert_eq!(stored["lastDay"], day(3));
        }

        async fn version(pool: &sqlx::PgPool) -> (i32, bool) {
            sqlx::query_as("SELECT version, changed_at > created_at FROM gatherings")
                .fetch_one(pool)
                .await
                .unwrap()
        }

        #[tokio::test]
        async fn each_change_raises_the_version() {
            let database = fresh_database().await;
            let pool = database.pool();
            let (_, cookie) = signed_in_organizer(&pool).await;
            let (_, created) =
                send(&pool, &cookie, "POST", "/api/gatherings", &weekend(10, 12)).await;
            assert_eq!(version(&pool).await, (0, false));
            let path = format!("/api/gatherings/{}", created["id"]);
            send(&pool, &cookie, "PUT", &path, &weekend(10, 13)).await;
            assert_eq!(version(&pool).await, (1, true));
            send(&pool, &cookie, "PUT", &path, &weekend(10, 13)).await;
            assert_eq!(version(&pool).await.0, 2);
        }

        #[tokio::test]
        async fn cancellation_raises_the_version() {
            let database = fresh_database().await;
            let pool = database.pool();
            let (_, cookie) = signed_in_organizer(&pool).await;
            let (_, created) =
                send(&pool, &cookie, "POST", "/api/gatherings", &weekend(10, 12)).await;
            let path = format!("/api/gatherings/{}/cancellation", created["id"]);
            let (status, _) = send(&pool, &cookie, "POST", &path, &Value::Null).await;
            assert_eq!(status, StatusCode::NO_CONTENT);
            assert_eq!(version(&pool).await, (1, true));
        }
    }
}
