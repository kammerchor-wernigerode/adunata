use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use sqlx::PgConnection;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::api::AppState;
use crate::localization;
use crate::validation::{ErrorCode, FieldError, field_error, invalid, is_unique_violation};

const MAX_NAME_LENGTH: usize = 100;
const MAX_MAIL_LENGTH: usize = 254;
const COLUMNS: &str = "id, first_name, last_name, mail, language, active, organizer";

/// A person in the ensemble whom organizers can invite.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Member {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub mail: String,
    /// Language of the member's mail, `de` or `en`.
    pub language: String,
    pub active: bool,
    pub organizer: bool,
}

/// What an organizer enters for a new member. A new member is active and holds no role.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NewMember {
    pub first_name: String,
    pub last_name: String,
    pub mail: String,
    pub language: String,
}

/// Every field of a member that an organizer can change.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MemberChange {
    pub first_name: String,
    pub last_name: String,
    pub mail: String,
    pub language: String,
    pub active: bool,
    pub organizer: bool,
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list_members, add_member))
        .routes(routes!(change_member))
}

#[utoipa::path(
    get,
    path = "/api/members",
    responses((status = 200, description = "Every member, sorted by last name", body = Vec<Member>))
)]
async fn list_members(State(state): State<AppState>) -> Response {
    let query =
        format!("SELECT {COLUMNS} FROM members ORDER BY lower(last_name), lower(first_name), id");
    match sqlx::query_as::<_, Member>(sqlx::AssertSqlSafe(query))
        .fetch_all(&state.pool)
        .await
    {
        Ok(members) => Json(members).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/members",
    request_body = NewMember,
    responses(
        (status = 201, description = "Member added as active", body = Member),
        (status = 422, description = "Member invalid", body = crate::validation::ValidationErrors),
    )
)]
async fn add_member(State(state): State<AppState>, Json(member): Json<NewMember>) -> Response {
    let member = MemberChange {
        first_name: member.first_name,
        last_name: member.last_name,
        mail: member.mail,
        language: member.language,
        active: true,
        organizer: false,
    };
    let errors = validate(&member);
    if !errors.is_empty() {
        return invalid(errors);
    }
    let Ok(mut connection) = state.pool.acquire().await else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    match insert(&mut connection, &member).await {
        Ok(stored) => (StatusCode::CREATED, Json(stored)).into_response(),
        Err(error) => rejected(&mut connection, &member.mail, &error).await,
    }
}

#[utoipa::path(
    put,
    path = "/api/members/{id}",
    params(("id" = i64, Path, description = "Id of the member")),
    request_body = MemberChange,
    responses(
        (status = 200, description = "Member changed", body = Member),
        (status = 404, description = "No such member"),
        (status = 422, description = "Change invalid or it would leave no active organizer", body = crate::validation::ValidationErrors),
    )
)]
async fn change_member(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(change): Json<MemberChange>,
) -> Response {
    let errors = validate(&change);
    if !errors.is_empty() {
        return invalid(errors);
    }
    match change_guarded(&state.pool, id, &change).await {
        Ok(Outcome::Done(stored)) => Json(stored).into_response(),
        Ok(Outcome::NotFound) => StatusCode::NOT_FOUND.into_response(),
        Ok(Outcome::LastOrganizer) => {
            let field = if change.organizer {
                "active"
            } else {
                "organizer"
            };
            invalid(vec![field_error(field, ErrorCode::LastOrganizer)])
        }
        Err(error) => match state.pool.acquire().await {
            Ok(mut connection) => rejected(&mut connection, &change.mail, &error).await,
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        },
    }
}

enum Outcome<T> {
    Done(T),
    NotFound,
    LastOrganizer,
}

async fn change_guarded(
    pool: &sqlx::PgPool,
    id: i64,
    change: &MemberChange,
) -> Result<Outcome<Member>, sqlx::Error> {
    let mut transaction = pool.begin().await?;
    if !(change.active && change.organizer) && is_last_organizer(&mut transaction, id).await? {
        return Ok(Outcome::LastOrganizer);
    }
    let Some(stored) = update(&mut transaction, id, change).await? else {
        return Ok(Outcome::NotFound);
    };
    transaction.commit().await?;
    Ok(Outcome::Done(stored))
}

/// Locks the rows of every active organizer and tells whether the member is the only one. Revoking the role, deactivating and deleting a member call it before they change anything, so two organizers who remove each other at once cannot leave the instance without one.
async fn is_last_organizer(connection: &mut PgConnection, id: i64) -> Result<bool, sqlx::Error> {
    let organizers: Vec<i64> = sqlx::query_scalar(
        "SELECT id FROM members WHERE active AND organizer ORDER BY id FOR UPDATE",
    )
    .fetch_all(connection)
    .await?;
    Ok(organizers == [id])
}

/// The active members, the only ones that new invitations reach.
#[cfg_attr(not(test), expect(dead_code, reason = "nobody is invited yet"))]
pub async fn active_members(connection: &mut PgConnection) -> Result<Vec<Member>, sqlx::Error> {
    let query = format!("SELECT {COLUMNS} FROM members WHERE active ORDER BY id");
    sqlx::query_as(sqlx::AssertSqlSafe(query))
        .fetch_all(connection)
        .await
}

async fn insert(
    connection: &mut PgConnection,
    member: &MemberChange,
) -> Result<Member, sqlx::Error> {
    let query = format!(
        "INSERT INTO members (first_name, last_name, mail, language, active, organizer) VALUES ($1, $2, $3, $4, $5, $6) RETURNING {COLUMNS}"
    );
    sqlx::query_as(sqlx::AssertSqlSafe(query))
        .bind(member.first_name.trim())
        .bind(member.last_name.trim())
        .bind(member.mail.trim())
        .bind(&member.language)
        .bind(member.active)
        .bind(member.organizer)
        .fetch_one(connection)
        .await
}

async fn update(
    connection: &mut PgConnection,
    id: i64,
    change: &MemberChange,
) -> Result<Option<Member>, sqlx::Error> {
    let query = format!(
        "UPDATE members SET first_name = $2, last_name = $3, mail = $4, language = $5, active = $6, organizer = $7 WHERE id = $1 RETURNING {COLUMNS}"
    );
    sqlx::query_as(sqlx::AssertSqlSafe(query))
        .bind(id)
        .bind(change.first_name.trim())
        .bind(change.last_name.trim())
        .bind(change.mail.trim())
        .bind(&change.language)
        .bind(change.active)
        .bind(change.organizer)
        .fetch_optional(connection)
        .await
}

/// Turns a failed write into a response. A taken mail address names the stored address it collides with.
async fn rejected(connection: &mut PgConnection, mail: &str, error: &sqlx::Error) -> Response {
    if !is_unique_violation(error) {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    let conflict: Option<String> =
        sqlx::query_scalar("SELECT mail FROM members WHERE lower(mail) = lower($1)")
            .bind(mail.trim())
            .fetch_optional(connection)
            .await
            .ok()
            .flatten();
    invalid(vec![FieldError {
        conflict,
        ..field_error("mail", ErrorCode::Taken)
    }])
}

fn validate(member: &MemberChange) -> Vec<FieldError> {
    let mut errors = Vec::new();
    for (field, value) in [
        ("firstName", &member.first_name),
        ("lastName", &member.last_name),
    ] {
        let length = value.trim().chars().count();
        if length == 0 {
            errors.push(field_error(field, ErrorCode::Required));
        } else if length > MAX_NAME_LENGTH {
            errors.push(field_error(field, ErrorCode::TooLong));
        }
    }
    let mail = member.mail.trim();
    if mail.is_empty() {
        errors.push(field_error("mail", ErrorCode::Required));
    } else if mail.chars().count() > MAX_MAIL_LENGTH {
        errors.push(field_error("mail", ErrorCode::TooLong));
    } else if mail.parse::<lettre::Address>().is_err() {
        errors.push(field_error("mail", ErrorCode::Invalid));
    }
    if !localization::is_supported(&member.language) {
        errors.push(field_error("language", ErrorCode::Invalid));
    }
    if member.organizer && !member.active {
        errors.push(field_error("organizer", ErrorCode::Invalid));
    }
    errors
}

#[cfg(test)]
mod tests {
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, header};
    use serde_json::{Value, json};
    use tower::ServiceExt;

    use super::*;
    use crate::api;
    use crate::database::testing::fresh_database;

    async fn send(
        pool: &sqlx::PgPool,
        method: &str,
        path: &str,
        body: &Value,
    ) -> (StatusCode, Value) {
        let request = Request::builder()
            .method(method)
            .uri(path)
            .header(header::CONTENT_TYPE, "application/json")
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

    fn ada() -> Value {
        json!({ "firstName": "Ada", "lastName": "Lovelace", "mail": "Ada@Example.org", "language": "en" })
    }

    #[tokio::test]
    async fn add_a_member() {
        let database = fresh_database().await;
        let pool = database.pool();
        let (status, body) = send(&pool, "POST", "/api/members", &ada()).await;
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(body["active"], true);
        assert_eq!(body["organizer"], false);
        let (_, list) = send(&pool, "GET", "/api/members", &Value::Null).await;
        assert_eq!(list.as_array().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn duplicate_address() {
        let database = fresh_database().await;
        let pool = database.pool();
        send(&pool, "POST", "/api/members", &ada()).await;
        let mut twin = ada();
        twin["mail"] = json!("ada@example.ORG");
        twin["firstName"] = json!("Augusta");
        let (status, body) = send(&pool, "POST", "/api/members", &twin).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            body,
            json!({ "errors": [{ "field": "mail", "code": "taken", "conflict": "Ada@Example.org" }] })
        );
    }

    #[tokio::test]
    async fn change_the_mail_address() {
        let database = fresh_database().await;
        let pool = database.pool();
        let (_, added) = send(&pool, "POST", "/api/members", &ada()).await;
        let mut change = added.clone();
        change["mail"] = json!("countess@example.org");
        let path = format!("/api/members/{}", added["id"]);
        let (status, body) = send(&pool, "PUT", &path, &change).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["mail"], "countess@example.org");
        let mut connection = pool.acquire().await.unwrap();
        let recipients: Vec<String> = active_members(&mut connection)
            .await
            .unwrap()
            .into_iter()
            .map(|member| member.mail)
            .collect();
        assert_eq!(recipients, vec!["countess@example.org"]);
    }

    #[tokio::test]
    async fn deactivate_a_member() {
        let database = fresh_database().await;
        let pool = database.pool();
        let (_, added) = send(&pool, "POST", "/api/members", &ada()).await;
        let mut change = added.clone();
        change["active"] = json!(false);
        let path = format!("/api/members/{}", added["id"]);
        let (status, _) = send(&pool, "PUT", &path, &change).await;
        assert_eq!(status, StatusCode::OK);
        let (_, list) = send(&pool, "GET", "/api/members", &Value::Null).await;
        assert_eq!(list[0]["active"], false);
        let mut connection = pool.acquire().await.unwrap();
        assert!(active_members(&mut connection).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn list_is_sorted_by_last_name() {
        let database = fresh_database().await;
        let pool = database.pool();
        for (first, last, mail) in [
            ("Clara", "schumann", "clara@example.org"),
            ("Johannes", "Brahms", "jb@example.org"),
        ] {
            send(
                &pool,
                "POST",
                "/api/members",
                &json!({ "firstName": first, "lastName": last, "mail": mail, "language": "de" }),
            )
            .await;
        }
        let (_, list) = send(&pool, "GET", "/api/members", &Value::Null).await;
        let names: Vec<&str> = list
            .as_array()
            .unwrap()
            .iter()
            .map(|member| member["lastName"].as_str().unwrap())
            .collect();
        assert_eq!(names, vec!["Brahms", "schumann"]);
    }

    #[tokio::test]
    async fn invalid_fields() {
        let database = fresh_database().await;
        let pool = database.pool();
        let (status, body) = send(
            &pool,
            "POST",
            "/api/members",
            &json!({ "firstName": " ", "lastName": "x".repeat(101), "mail": "no-address", "language": "fr" }),
        )
        .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            body,
            json!({ "errors": [
                { "field": "firstName", "code": "required" },
                { "field": "lastName", "code": "tooLong" },
                { "field": "mail", "code": "invalid" },
                { "field": "language", "code": "invalid" },
            ] })
        );
    }

    async fn organizer(pool: &sqlx::PgPool, first_name: &str) -> Value {
        let (_, added) = send(
            pool,
            "POST",
            "/api/members",
            &json!({ "firstName": first_name, "lastName": "Organizer", "mail": format!("{first_name}@example.org"), "language": "de" }),
        )
        .await;
        let mut change = added.clone();
        change["organizer"] = json!(true);
        let (status, stored) = send(
            pool,
            "PUT",
            &format!("/api/members/{}", added["id"]),
            &change,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        stored
    }

    async fn active_organizers(pool: &sqlx::PgPool) -> i64 {
        sqlx::query_scalar("SELECT count(*) FROM members WHERE active AND organizer")
            .fetch_one(pool)
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn revoke_the_last_organizer() {
        let database = fresh_database().await;
        let pool = database.pool();
        let mut only = organizer(&pool, "ada").await;
        only["organizer"] = json!(false);
        let (status, body) =
            send(&pool, "PUT", &format!("/api/members/{}", only["id"]), &only).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            body,
            json!({ "errors": [{ "field": "organizer", "code": "lastOrganizer" }] })
        );
        assert_eq!(active_organizers(&pool).await, 1);
    }

    #[tokio::test]
    async fn deactivate_the_last_organizer() {
        let database = fresh_database().await;
        let pool = database.pool();
        let mut only = organizer(&pool, "ada").await;
        only["active"] = json!(false);
        only["organizer"] = json!(false);
        let (status, _) = send(&pool, "PUT", &format!("/api/members/{}", only["id"]), &only).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(active_organizers(&pool).await, 1);
    }

    #[tokio::test]
    async fn revoke_one_of_two_organizers() {
        let database = fresh_database().await;
        let pool = database.pool();
        organizer(&pool, "ada").await;
        let mut second = organizer(&pool, "clara").await;
        second["organizer"] = json!(false);
        let (status, _) = send(
            &pool,
            "PUT",
            &format!("/api/members/{}", second["id"]),
            &second,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(active_organizers(&pool).await, 1);
    }

    fn revoked(member: &Value) -> MemberChange {
        MemberChange {
            organizer: false,
            ..serde_json::from_value(member.clone()).unwrap()
        }
    }

    #[tokio::test]
    async fn two_concurrent_revocations_leave_one_organizer() {
        let database = fresh_database().await;
        let pool = database.pool();
        let first = organizer(&pool, "ada").await;
        let second = organizer(&pool, "clara").await;
        let first_id = first["id"].as_i64().unwrap();
        let second_id = second["id"].as_i64().unwrap();

        let mut transaction = pool.begin().await.unwrap();
        assert!(!is_last_organizer(&mut transaction, first_id).await.unwrap());
        let other = tokio::spawn({
            let pool = pool.clone();
            let change = revoked(&second);
            async move { change_guarded(&pool, second_id, &change).await }
        });
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        update(&mut transaction, first_id, &revoked(&first))
            .await
            .unwrap();
        transaction.commit().await.unwrap();

        let outcome = other.await.unwrap().unwrap();
        assert!(matches!(outcome, Outcome::LastOrganizer));
        assert_eq!(active_organizers(&pool).await, 1);
    }

    #[tokio::test]
    async fn role_only_for_active_members() {
        let database = fresh_database().await;
        let pool = database.pool();
        let (_, added) = send(&pool, "POST", "/api/members", &ada()).await;
        let mut change = added.clone();
        change["active"] = json!(false);
        change["organizer"] = json!(true);
        let (status, body) = send(
            &pool,
            "PUT",
            &format!("/api/members/{}", added["id"]),
            &change,
        )
        .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            body,
            json!({ "errors": [{ "field": "organizer", "code": "invalid" }] })
        );
    }

    #[tokio::test]
    async fn change_an_unknown_member() {
        let database = fresh_database().await;
        let pool = database.pool();
        let mut change = ada();
        change["active"] = json!(true);
        change["organizer"] = json!(false);
        let (status, _) = send(&pool, "PUT", "/api/members/999", &change).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }
}
