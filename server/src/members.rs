use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use sqlx::PgConnection;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::access::Organizer;
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
        .routes(routes!(change_member, delete_member))
}

#[utoipa::path(
    get,
    path = "/api/members",
    responses(
        (status = 200, description = "Every member, sorted by last name", body = Vec<Member>),
        (status = 401, description = "No valid organizer session"),
    )
)]
async fn list_members(_: Organizer, State(state): State<AppState>) -> Response {
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
        (status = 401, description = "No valid organizer session"),
        (status = 422, description = "Member invalid", body = crate::validation::ValidationErrors),
    )
)]
async fn add_member(
    _: Organizer,
    State(state): State<AppState>,
    Json(member): Json<NewMember>,
) -> Response {
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
        (status = 401, description = "No valid organizer session"),
        (status = 404, description = "No such member"),
        (status = 422, description = "Change invalid or it would leave no active organizer", body = crate::validation::ValidationErrors),
    )
)]
async fn change_member(
    _: Organizer,
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

#[utoipa::path(
    delete,
    path = "/api/members/{id}",
    params(("id" = i64, Path, description = "Id of the member")),
    responses(
        (status = 204, description = "Member deleted together with everything that belongs to the member"),
        (status = 401, description = "No valid organizer session"),
        (status = 404, description = "No such member"),
        (status = 422, description = "The member is the last active organizer", body = crate::validation::ValidationErrors),
    )
)]
async fn delete_member(
    _: Organizer,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Response {
    match delete_guarded(&state.pool, id).await {
        Ok(Outcome::Done(())) => StatusCode::NO_CONTENT.into_response(),
        Ok(Outcome::NotFound) => StatusCode::NOT_FOUND.into_response(),
        Ok(Outcome::LastOrganizer) => {
            invalid(vec![field_error("member", ErrorCode::LastOrganizer)])
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// Deletes a member. Foreign keys delete everything that references the member in the same statement.
async fn delete_guarded(pool: &sqlx::PgPool, id: i64) -> Result<Outcome<()>, sqlx::Error> {
    let mut transaction = pool.begin().await?;
    if is_last_organizer(&mut transaction, id).await? {
        return Ok(Outcome::LastOrganizer);
    }
    let deleted = sqlx::query("DELETE FROM members WHERE id = $1")
        .bind(id)
        .execute(&mut *transaction)
        .await?
        .rows_affected();
    if deleted == 0 {
        return Ok(Outcome::NotFound);
    }
    transaction.commit().await?;
    Ok(Outcome::Done(()))
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

/// The active organizer with this mail address, ignoring case.
pub async fn active_organizer_by_mail(
    connection: &mut PgConnection,
    mail: &str,
) -> Result<Option<Member>, sqlx::Error> {
    let query = format!(
        "SELECT {COLUMNS} FROM members WHERE lower(mail) = lower($1) AND active AND organizer"
    );
    sqlx::query_as(sqlx::AssertSqlSafe(query))
        .bind(mail.trim())
        .fetch_optional(connection)
        .await
}

/// The member with this id if the member is active and holds the organizer role.
pub async fn active_organizer(
    connection: &mut PgConnection,
    id: i64,
) -> Result<Option<Member>, sqlx::Error> {
    let query = format!("SELECT {COLUMNS} FROM members WHERE id = $1 AND active AND organizer");
    sqlx::query_as(sqlx::AssertSqlSafe(query))
        .bind(id)
        .fetch_optional(connection)
        .await
}

/// Whether any active member holds the organizer role.
pub async fn has_active_organizer(connection: &mut PgConnection) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM members WHERE active AND organizer)")
        .fetch_one(connection)
        .await
}

/// Makes the member with this mail address an active organizer. Creates the member with the given names and language if no member has the address in any case.
pub async fn grant_organizer(
    connection: &mut PgConnection,
    mail: &str,
    first_name: &str,
    last_name: &str,
    language: &str,
) -> Result<Member, sqlx::Error> {
    let query = format!(
        "INSERT INTO members (first_name, last_name, mail, language, active, organizer) VALUES ($1, $2, $3, $4, true, true) ON CONFLICT ((lower(mail))) DO UPDATE SET active = true, organizer = true RETURNING {COLUMNS}"
    );
    sqlx::query_as(sqlx::AssertSqlSafe(query))
        .bind(first_name.trim())
        .bind(last_name.trim())
        .bind(mail.trim())
        .bind(language)
        .fetch_one(connection)
        .await
}

/// The active members, the only ones that new invitations reach.
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
    use sqlx::PgPool;
    use tower::ServiceExt;

    use super::*;
    use crate::access::testing::signed_in_organizer;
    use crate::api;
    use crate::database::testing::{FreshDatabase, fresh_database};

    /// A test database with one signed-in organizer, Root Organizer, who sends every request.
    struct Instance {
        _database: FreshDatabase,
        pool: PgPool,
        root: i64,
        cookie: String,
    }

    async fn instance() -> Instance {
        let database = fresh_database().await;
        let pool = database.pool();
        let (root, cookie) = signed_in_organizer(&pool).await;
        Instance {
            _database: database,
            pool,
            root,
            cookie,
        }
    }

    impl Instance {
        async fn send(&self, method: &str, path: &str, body: &Value) -> (StatusCode, Value) {
            let request = Request::builder()
                .method(method)
                .uri(path)
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, &self.cookie)
                .body(Body::from(body.to_string()))
                .unwrap();
            let response = api::router(api::testing::state(self.pool.clone()))
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

        async fn add(&self, member: &Value) -> Value {
            let (status, added) = self.send("POST", "/api/members", member).await;
            assert_eq!(status, StatusCode::CREATED, "{added}");
            added
        }

        async fn change(&self, member: &Value) -> (StatusCode, Value) {
            self.send("PUT", &format!("/api/members/{}", member["id"]), member)
                .await
        }

        async fn root(&self) -> Value {
            let (_, list) = self.send("GET", "/api/members", &Value::Null).await;
            list.as_array()
                .unwrap()
                .iter()
                .find(|member| member["id"] == self.root)
                .unwrap()
                .clone()
        }

        async fn organizer(&self, first_name: &str) -> Value {
            let added = self
                .add(&json!({ "firstName": first_name, "lastName": "Organizer", "mail": format!("{first_name}@example.org"), "language": "de" }))
                .await;
            let mut change = added.clone();
            change["organizer"] = json!(true);
            let (status, stored) = self.change(&change).await;
            assert_eq!(status, StatusCode::OK);
            stored
        }

        async fn active_organizers(&self) -> i64 {
            sqlx::query_scalar("SELECT count(*) FROM members WHERE active AND organizer")
                .fetch_one(&self.pool)
                .await
                .unwrap()
        }

        async fn active_mail_addresses(&self) -> Vec<String> {
            let mut connection = self.pool.acquire().await.unwrap();
            active_members(&mut connection)
                .await
                .unwrap()
                .into_iter()
                .map(|member| member.mail)
                .collect()
        }
    }

    fn ada() -> Value {
        json!({ "firstName": "Ada", "lastName": "Lovelace", "mail": "Ada@Example.org", "language": "en" })
    }

    #[tokio::test]
    async fn request_without_a_session() {
        let instance = instance().await;
        let request = Request::get("/api/members").body(Body::empty()).unwrap();
        let response = api::router(api::testing::state(instance.pool.clone()))
            .oneshot(request)
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn add_a_member() {
        let instance = instance().await;
        let added = instance.add(&ada()).await;
        assert_eq!(added["active"], true);
        assert_eq!(added["organizer"], false);
        assert!(
            instance
                .active_mail_addresses()
                .await
                .contains(&"Ada@Example.org".to_owned())
        );
    }

    #[tokio::test]
    async fn duplicate_address() {
        let instance = instance().await;
        instance.add(&ada()).await;
        let mut twin = ada();
        twin["mail"] = json!("ada@example.ORG");
        twin["firstName"] = json!("Augusta");
        let (status, body) = instance.send("POST", "/api/members", &twin).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            body,
            json!({ "errors": [{ "field": "mail", "code": "taken", "conflict": "Ada@Example.org" }] })
        );
    }

    #[tokio::test]
    async fn change_the_mail_address() {
        let instance = instance().await;
        let mut change = instance.add(&ada()).await;
        change["mail"] = json!("countess@example.org");
        let (status, body) = instance.change(&change).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["mail"], "countess@example.org");
        let addresses = instance.active_mail_addresses().await;
        assert!(addresses.contains(&"countess@example.org".to_owned()));
        assert!(!addresses.contains(&"Ada@Example.org".to_owned()));
    }

    #[tokio::test]
    async fn deactivate_a_member() {
        let instance = instance().await;
        let mut change = instance.add(&ada()).await;
        change["active"] = json!(false);
        let (status, _) = instance.change(&change).await;
        assert_eq!(status, StatusCode::OK);
        let (_, list) = instance.send("GET", "/api/members", &Value::Null).await;
        assert!(
            list.as_array()
                .unwrap()
                .iter()
                .any(|member| member["id"] == change["id"] && member["active"] == false)
        );
        assert!(
            !instance
                .active_mail_addresses()
                .await
                .contains(&"Ada@Example.org".to_owned())
        );
    }

    #[tokio::test]
    async fn list_is_sorted_by_last_name() {
        let instance = instance().await;
        for (first, last, mail) in [
            ("Clara", "schumann", "clara@example.org"),
            ("Johannes", "Brahms", "jb@example.org"),
        ] {
            instance
                .add(&json!({ "firstName": first, "lastName": last, "mail": mail, "language": "de" }))
                .await;
        }
        let (_, list) = instance.send("GET", "/api/members", &Value::Null).await;
        let names: Vec<&str> = list
            .as_array()
            .unwrap()
            .iter()
            .map(|member| member["lastName"].as_str().unwrap())
            .collect();
        assert_eq!(names, vec!["Brahms", "Organizer", "schumann"]);
    }

    #[tokio::test]
    async fn invalid_fields() {
        let instance = instance().await;
        let (status, body) = instance
            .send(
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

    #[tokio::test]
    async fn revoke_the_last_organizer() {
        let instance = instance().await;
        let mut only = instance.root().await;
        only["organizer"] = json!(false);
        let (status, body) = instance.change(&only).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            body,
            json!({ "errors": [{ "field": "organizer", "code": "lastOrganizer" }] })
        );
        assert_eq!(instance.active_organizers().await, 1);
    }

    #[tokio::test]
    async fn deactivate_the_last_organizer() {
        let instance = instance().await;
        let mut only = instance.root().await;
        only["active"] = json!(false);
        only["organizer"] = json!(false);
        let (status, _) = instance.change(&only).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(instance.active_organizers().await, 1);
    }

    #[tokio::test]
    async fn revoke_one_of_two_organizers() {
        let instance = instance().await;
        let mut second = instance.organizer("clara").await;
        second["organizer"] = json!(false);
        let (status, _) = instance.change(&second).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(instance.active_organizers().await, 1);
    }

    fn revoked(member: &Value) -> MemberChange {
        MemberChange {
            organizer: false,
            ..serde_json::from_value(member.clone()).unwrap()
        }
    }

    #[tokio::test]
    async fn two_concurrent_revocations_leave_one_organizer() {
        let instance = instance().await;
        let first = instance.root().await;
        let second = instance.organizer("clara").await;
        let pool = instance.pool.clone();
        let second_id = second["id"].as_i64().unwrap();

        let mut transaction = pool.begin().await.unwrap();
        assert!(
            !is_last_organizer(&mut transaction, instance.root)
                .await
                .unwrap()
        );
        let other = tokio::spawn({
            let pool = pool.clone();
            let change = revoked(&second);
            async move { change_guarded(&pool, second_id, &change).await }
        });
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        update(&mut transaction, instance.root, &revoked(&first))
            .await
            .unwrap();
        transaction.commit().await.unwrap();

        let outcome = other.await.unwrap().unwrap();
        assert!(matches!(outcome, Outcome::LastOrganizer));
        assert_eq!(instance.active_organizers().await, 1);
    }

    #[tokio::test]
    async fn role_only_for_active_members() {
        let instance = instance().await;
        let mut change = instance.add(&ada()).await;
        change["active"] = json!(false);
        change["organizer"] = json!(true);
        let (status, body) = instance.change(&change).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            body,
            json!({ "errors": [{ "field": "organizer", "code": "invalid" }] })
        );
    }

    #[tokio::test]
    async fn delete_the_last_organizer() {
        let instance = instance().await;
        let (status, body) = instance
            .send(
                "DELETE",
                &format!("/api/members/{}", instance.root),
                &Value::Null,
            )
            .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            body,
            json!({ "errors": [{ "field": "member", "code": "lastOrganizer" }] })
        );
        assert_eq!(instance.active_organizers().await, 1);
    }

    async fn count(pool: &PgPool, table: &str, member_id: i64) -> i64 {
        sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
            "SELECT count(*) FROM {table} WHERE member_id = $1"
        )))
        .bind(member_id)
        .fetch_one(pool)
        .await
        .unwrap()
    }

    /// Covers the scenarios "Delete a member" and "Member deleted while a mail waits". Each table that references a member adds its rows here.
    #[tokio::test]
    async fn delete_a_member_with_everything_that_belongs_to_it() {
        let instance = instance().await;
        let clara = instance.organizer("clara").await;
        let id = clara["id"].as_i64().unwrap();
        let pool = &instance.pool;
        sqlx::query("INSERT INTO mail_outbox (recipient, member_id, subject, text_body, html_body) VALUES ('clara@example.org', $1, 's', 't', 'h')")
            .bind(id)
            .execute(pool)
            .await
            .unwrap();
        crate::access::testing::session_cookie(pool, id).await;
        sqlx::query("INSERT INTO sign_in_links (member_id, secret_hash, expires_at) VALUES ($1, '\\x01', now())")
            .bind(id)
            .execute(pool)
            .await
            .unwrap();

        let gathering =
            crate::invitations::testing::gathering(pool, "Probenwochenende", 10, 12).await;
        sqlx::query("INSERT INTO invitations (gathering_id, member_id, secret_hash) VALUES ($1, $2, '\\x02')")
            .bind(gathering)
            .bind(id)
            .execute(pool)
            .await
            .unwrap();

        let (status, _) = instance
            .send("DELETE", &format!("/api/members/{id}"), &Value::Null)
            .await;
        assert_eq!(status, StatusCode::NO_CONTENT);
        let (_, list) = instance.send("GET", "/api/members", &Value::Null).await;
        assert!(
            list.as_array()
                .unwrap()
                .iter()
                .all(|member| member["id"] != id)
        );
        for table in ["mail_outbox", "sessions", "sign_in_links", "invitations"] {
            assert_eq!(count(pool, table, id).await, 0, "{table}");
        }
    }

    #[tokio::test]
    async fn delete_an_unknown_member() {
        let instance = instance().await;
        let (status, _) = instance
            .send("DELETE", "/api/members/999", &Value::Null)
            .await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn change_an_unknown_member() {
        let instance = instance().await;
        let mut change = ada();
        change["active"] = json!(true);
        change["organizer"] = json!(false);
        let (status, _) = instance.send("PUT", "/api/members/999", &change).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }
}
