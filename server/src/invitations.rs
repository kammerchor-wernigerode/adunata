use std::fmt::Write;

use axum::Json;
use axum::extract::{FromRequestParts, Path, State};
use axum::http::StatusCode;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use fluent_bundle::FluentArgs;
use serde::{Deserialize, Serialize};
use sqlx::PgConnection;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::access::Organizer;
use crate::api::AppState;
use crate::configuration::{LegalLinks, PublicUrl};
use crate::gatherings::{self, Gathering};
use crate::localization;
use crate::mail::{self, Mail};
use crate::members::{self, Member};
use crate::secret;
use crate::validation::{ErrorCode, field_error, invalid};

/// An invitation opened through its personal link. The `responses` module takes it as an argument, so a request without a valid secret in `Authorization: Bearer` fails with status 404 before any handler runs. An unknown and a revoked secret fail alike and reveal no gathering or member.
pub struct PersonalLink {
    pub invitation_id: i64,
    pub gathering: Gathering,
    pub member: Member,
}

impl FromRequestParts<AppState> for PersonalLink {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let secret = secret::from_authorization(&parts.headers).ok_or(StatusCode::NOT_FOUND)?;
        let mut connection = state
            .pool
            .acquire()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        resolve(&mut connection, secret, state.time_zone.today())
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)
    }
}

async fn resolve(
    connection: &mut PgConnection,
    secret: &str,
    today: time::Date,
) -> Result<Option<PersonalLink>, sqlx::Error> {
    let found: Option<(i64, i64, i64)> = sqlx::query_as(
        "SELECT id, gathering_id, member_id FROM invitations WHERE secret_hash = $1",
    )
    .bind(secret::hash(secret))
    .fetch_optional(&mut *connection)
    .await?;
    let Some((invitation_id, gathering_id, member_id)) = found else {
        return Ok(None);
    };
    let gathering = gatherings::find(connection, gathering_id, today).await?;
    let member = members::find(connection, member_id).await?;
    Ok(gathering
        .zip(member)
        .map(|(gathering, member)| PersonalLink {
            invitation_id,
            gathering,
            member,
        }))
}

/// Deletes the invitations of every gathering whose last day lies before the given date, together with their responses through the foreign key. The gatherings stay.
pub async fn delete_of_gatherings_ended_before(
    connection: &mut PgConnection,
    date: time::Date,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM invitations USING gatherings WHERE gatherings.id = invitations.gathering_id AND gatherings.last_day < $1")
        .bind(date)
        .execute(connection)
        .await
        .map(|_| ())
}

/// The gathering of an invitation, judged against the given date.
pub async fn gathering_of(
    connection: &mut PgConnection,
    invitation_id: i64,
    today: time::Date,
) -> Result<Option<Gathering>, sqlx::Error> {
    let gathering_id: Option<i64> =
        sqlx::query_scalar("SELECT gathering_id FROM invitations WHERE id = $1")
            .bind(invitation_id)
            .fetch_optional(&mut *connection)
            .await?;
    match gathering_id {
        Some(id) => gatherings::find(connection, id, today).await,
        None => Ok(None),
    }
}

/// Which members to invite. Without ids Adunata invites every active member who is not invited yet.
#[derive(Clone, Debug, Default, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct InvitationRequest {
    #[serde(default)]
    pub member_ids: Option<Vec<i64>>,
}

/// How many members Adunata invited.
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct Invited {
    pub count: usize,
}

/// Where the newest mail of an invitation stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[serde(rename_all = "camelCase")]
#[sqlx(type_name = "text", rename_all = "lowercase")]
pub enum MailState {
    Waiting,
    Sent,
    Failed,
}

/// One invitation as organizers manage it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Invitation {
    pub id: i64,
    pub member_id: i64,
    pub first_name: String,
    pub last_name: String,
    /// Whether the personal link works. Revoking clears it until the next resend.
    pub link_active: bool,
    pub mail: Option<MailState>,
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list_invitations, invite))
        .routes(routes!(resend))
        .routes(routes!(revoke))
        .routes(routes!(cancel))
}

#[utoipa::path(
    post,
    path = "/api/gatherings/{id}/cancellation",
    params(("id" = i64, Path, description = "Id of the gathering")),
    responses(
        (status = 204, description = "Gathering cancelled and a mail queued for every invited member"),
        (status = 401, description = "No valid organizer session"),
        (status = 404, description = "No such gathering"),
        (status = 422, description = "The gathering is cancelled already or has ended", body = crate::validation::ValidationErrors),
    )
)]
async fn cancel(_: Organizer, State(state): State<AppState>, Path(id): Path<i64>) -> Response {
    match cancel_gathering(&state, id).await {
        Ok(Outcome::Done(())) => StatusCode::NO_CONTENT.into_response(),
        Ok(Outcome::NotFound) => StatusCode::NOT_FOUND.into_response(),
        Ok(Outcome::Closed(code)) => invalid(vec![field_error("gathering", code)]),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// Cancels a gathering and queues a mail to every invited member in one transaction. This module owns the action, because it knows the recipients.
async fn cancel_gathering(state: &AppState, id: i64) -> Result<Outcome<()>, sqlx::Error> {
    let today = state.time_zone.today();
    let mut transaction = state.pool.begin().await?;
    let Some(gathering) = gatherings::cancel(&mut transaction, id, today).await? else {
        return Ok(match gatherings::find(&mut transaction, id, today).await? {
            None => Outcome::NotFound,
            Some(gathering) if gathering.cancelled => Outcome::Closed(ErrorCode::Cancelled),
            Some(_) => Outcome::Closed(ErrorCode::Ended),
        });
    };
    let invited: Vec<(i64, i64)> =
        sqlx::query_as("SELECT id, member_id FROM invitations WHERE gathering_id = $1 ORDER BY id")
            .bind(id)
            .fetch_all(&mut *transaction)
            .await?;
    for (invitation_id, member_id) in invited {
        if let Some(member) = members::find(&mut transaction, member_id).await? {
            mail::enqueue(
                &mut transaction,
                &cancellation_mail(&member, &gathering, invitation_id),
            )
            .await?;
        }
    }
    transaction.commit().await?;
    Ok(Outcome::Done(()))
}

fn cancellation_mail(member: &Member, gathering: &Gathering, invitation_id: i64) -> Mail {
    let localizer = localization::localizer();
    let language = member.language.as_str();
    let mut args = FluentArgs::new();
    args.set("firstName", member.first_name.clone());
    args.set("title", gathering.title.clone());
    args.set("location", gathering.location.clone());
    args.set(
        "span",
        localizer.date_span(language, gathering.first_day, gathering.last_day),
    );
    let greeting = localizer.message(language, "invitation-mail-greeting", &args);
    let body = localizer.message(language, "cancellation-mail-body", &args);
    Mail {
        recipient: member.mail.clone(),
        member_id: Some(member.id),
        invitation_id: Some(invitation_id),
        subject: localizer.message(language, "cancellation-mail-subject", &args),
        text: format!("{greeting}\n\n{body}\n"),
        html: format!(
            "<p>{}</p><p>{}</p>",
            mail::escape_html(&greeting),
            mail::escape_html(&body)
        ),
    }
}

#[utoipa::path(
    get,
    path = "/api/gatherings/{id}/invitations",
    params(("id" = i64, Path, description = "Id of the gathering")),
    responses(
        (status = 200, description = "Every invitation of the gathering, sorted by last name", body = Vec<Invitation>),
        (status = 401, description = "No valid organizer session"),
    )
)]
async fn list_invitations(
    _: Organizer,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Response {
    let Ok(mut connection) = state.pool.acquire().await else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    match of_gathering(&mut connection, id).await {
        Ok(invitations) => Json(invitations).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// Every invitation of a gathering with its member and the state of its newest mail.
pub async fn of_gathering(
    connection: &mut PgConnection,
    gathering_id: i64,
) -> Result<Vec<Invitation>, sqlx::Error> {
    sqlx::query_as(
        "SELECT invitations.id, members.id AS member_id, members.first_name, members.last_name, invitations.secret_hash IS NOT NULL AS link_active, (SELECT CASE WHEN sent_at IS NOT NULL THEN 'sent' WHEN failed_at IS NOT NULL THEN 'failed' ELSE 'waiting' END FROM mail_outbox WHERE invitation_id = invitations.id ORDER BY id DESC LIMIT 1) AS mail FROM invitations JOIN members ON members.id = invitations.member_id WHERE gathering_id = $1 ORDER BY lower(members.last_name), lower(members.first_name), invitations.id",
    )
    .bind(gathering_id)
    .fetch_all(connection)
    .await
}

#[utoipa::path(
    post,
    path = "/api/invitations/{id}/resend",
    params(("id" = i64, Path, description = "Id of the invitation")),
    responses(
        (status = 204, description = "New link created, the old one revoked and the mail queued"),
        (status = 401, description = "No valid organizer session"),
        (status = 404, description = "No such invitation"),
        (status = 422, description = "The gathering is cancelled or has ended", body = crate::validation::ValidationErrors),
    )
)]
async fn resend(_: Organizer, State(state): State<AppState>, Path(id): Path<i64>) -> Response {
    match resend_invitation(&state, id).await {
        Ok(Outcome::Done(())) => StatusCode::NO_CONTENT.into_response(),
        Ok(Outcome::NotFound) => StatusCode::NOT_FOUND.into_response(),
        Ok(Outcome::Closed(code)) => invalid(vec![field_error("gathering", code)]),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// Stores the hash of a new secret in place of the old one, so the old link stops working in the same transaction, and queues a mail with the new link.
async fn resend_invitation(state: &AppState, id: i64) -> Result<Outcome<()>, sqlx::Error> {
    let mut transaction = state.pool.begin().await?;
    let found: Option<(i64, i64)> =
        sqlx::query_as("SELECT gathering_id, member_id FROM invitations WHERE id = $1 FOR UPDATE")
            .bind(id)
            .fetch_optional(&mut *transaction)
            .await?;
    let Some((gathering_id, member_id)) = found else {
        return Ok(Outcome::NotFound);
    };
    let gathering = match open_gathering(&mut transaction, gathering_id, state).await? {
        Ok(gathering) => gathering,
        Err(Outcome::Closed(code)) => return Ok(Outcome::Closed(code)),
        Err(_) => return Ok(Outcome::NotFound),
    };
    let Some(member) = members::find(&mut transaction, member_id).await? else {
        return Ok(Outcome::NotFound);
    };
    let secret = secret::generate();
    sqlx::query("UPDATE invitations SET secret_hash = $2 WHERE id = $1")
        .bind(id)
        .bind(secret::hash(&secret))
        .execute(&mut *transaction)
        .await?;
    let link = personal_link(&state.public_url, &secret);
    mail::enqueue(
        &mut transaction,
        &invitation_mail(&member, &gathering, &link, &state.legal_links, id),
    )
    .await?;
    transaction.commit().await?;
    Ok(Outcome::Done(()))
}

#[utoipa::path(
    post,
    path = "/api/invitations/{id}/revocation",
    params(("id" = i64, Path, description = "Id of the invitation")),
    responses(
        (status = 204, description = "The link no longer works, and no mail goes out"),
        (status = 401, description = "No valid organizer session"),
        (status = 404, description = "No such invitation"),
    )
)]
async fn revoke(_: Organizer, State(state): State<AppState>, Path(id): Path<i64>) -> StatusCode {
    match sqlx::query("UPDATE invitations SET secret_hash = NULL WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await
    {
        Ok(result) if result.rows_affected() == 0 => StatusCode::NOT_FOUND,
        Ok(_) => StatusCode::NO_CONTENT,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

#[utoipa::path(
    post,
    path = "/api/gatherings/{id}/invitations",
    params(("id" = i64, Path, description = "Id of the gathering")),
    request_body = InvitationRequest,
    responses(
        (status = 201, description = "Members invited and their mail queued", body = Invited),
        (status = 401, description = "No valid organizer session"),
        (status = 404, description = "No such gathering"),
        (status = 422, description = "The gathering is cancelled or has ended", body = crate::validation::ValidationErrors),
    )
)]
async fn invite(
    _: Organizer,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(request): Json<InvitationRequest>,
) -> Response {
    match invite_members(&state, id, request.member_ids.as_deref()).await {
        Ok(Outcome::Done(count)) => (StatusCode::CREATED, Json(Invited { count })).into_response(),
        Ok(Outcome::NotFound) => StatusCode::NOT_FOUND.into_response(),
        Ok(Outcome::Closed(code)) => invalid(vec![field_error("gathering", code)]),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

enum Outcome<T> {
    Done(T),
    NotFound,
    Closed(ErrorCode),
}

/// Finds a gathering that still takes invitations, or tells why it does not.
async fn open_gathering(
    connection: &mut PgConnection,
    id: i64,
    state: &AppState,
) -> Result<Result<Gathering, Outcome<usize>>, sqlx::Error> {
    let Some(gathering) = gatherings::find(connection, id, state.time_zone.today()).await? else {
        return Ok(Err(Outcome::NotFound));
    };
    if gathering.cancelled {
        return Ok(Err(Outcome::Closed(ErrorCode::Cancelled)));
    }
    if gathering.ended {
        return Ok(Err(Outcome::Closed(ErrorCode::Ended)));
    }
    Ok(Ok(gathering))
}

async fn invite_members(
    state: &AppState,
    gathering_id: i64,
    chosen: Option<&[i64]>,
) -> Result<Outcome<usize>, sqlx::Error> {
    let mut transaction = state.pool.begin().await?;
    let gathering = match open_gathering(&mut transaction, gathering_id, state).await? {
        Ok(gathering) => gathering,
        Err(outcome) => return Ok(outcome),
    };
    let candidates = members::active_members(&mut transaction)
        .await?
        .into_iter()
        .filter(|member| chosen.is_none_or(|ids| ids.contains(&member.id)));
    let mut count = 0;
    for member in candidates {
        let secret = secret::generate();
        let invitation_id: Option<i64> = sqlx::query_scalar(
            "INSERT INTO invitations (gathering_id, member_id, secret_hash) VALUES ($1, $2, $3) ON CONFLICT (gathering_id, member_id) DO NOTHING RETURNING id",
        )
        .bind(gathering.id)
        .bind(member.id)
        .bind(secret::hash(&secret))
        .fetch_optional(&mut *transaction)
        .await?;
        let Some(invitation_id) = invitation_id else {
            continue;
        };
        let link = personal_link(&state.public_url, &secret);
        let mail = invitation_mail(
            &member,
            &gathering,
            &link,
            &state.legal_links,
            invitation_id,
        );
        mail::enqueue(&mut transaction, &mail).await?;
        count += 1;
    }
    transaction.commit().await?;
    Ok(Outcome::Done(count))
}

/// The personal link of an invitation. The secret sits in the fragment, which browsers never send to a server.
fn personal_link(public_url: &PublicUrl, secret: &str) -> String {
    public_url.link(&format!("/invitation#{secret}"))
}

fn invitation_mail(
    member: &Member,
    gathering: &Gathering,
    link: &str,
    legal_links: &LegalLinks,
    invitation_id: i64,
) -> Mail {
    let localizer = localization::localizer();
    let language = member.language.as_str();
    let mut args = FluentArgs::new();
    args.set("firstName", member.first_name.clone());
    args.set("title", gathering.title.clone());
    args.set("location", gathering.location.clone());
    args.set(
        "span",
        localizer.date_span(language, gathering.first_day, gathering.last_day),
    );
    let subject = localizer.message(language, "invitation-mail-subject", &args);
    let greeting = localizer.message(language, "invitation-mail-greeting", &args);
    let details = localizer.message(language, "invitation-mail-details", &args);
    let answer = localizer.message(language, "invitation-mail-answer", &args);
    let mut text = format!("{greeting}\n\n{details}\n\n{answer}\n{link}\n");
    let mut html = format!(
        "<p>{}</p><p>{}</p><p>{}<br><a href=\"{link}\">{link}</a></p>",
        mail::escape_html(&greeting),
        mail::escape_html(&details),
        mail::escape_html(&answer)
    );
    if let Some(description) = &gathering.description {
        write!(text, "\n{description}\n").expect("writing to a string cannot fail");
        write!(
            html,
            "<p>{}</p>",
            mail::escape_html(description).replace('\n', "<br>")
        )
        .expect("writing to a string cannot fail");
    }
    let footer: Vec<(String, &String)> = [
        ("privacy-notice", &legal_links.privacy),
        ("imprint", &legal_links.imprint),
    ]
    .into_iter()
    .filter_map(|(id, url)| {
        url.as_ref()
            .map(|url| (localizer.message(language, id, &FluentArgs::new()), url))
    })
    .collect();
    if !footer.is_empty() {
        text.push_str("\n--\n");
        html.push_str("<hr>");
        for (label, url) in footer {
            writeln!(text, "{label}: {url}").expect("writing to a string cannot fail");
            write!(
                html,
                "<p><a href=\"{}\">{}</a></p>",
                mail::escape_html(url),
                mail::escape_html(&label)
            )
            .expect("writing to a string cannot fail");
        }
    }
    Mail {
        recipient: member.mail.clone(),
        member_id: Some(member.id),
        invitation_id: Some(invitation_id),
        subject,
        text,
        html,
    }
}

#[cfg(test)]
pub mod testing {
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode, header};
    use serde_json::Value;
    use sqlx::PgPool;
    use tower::ServiceExt;

    use crate::api::{self, AppState};

    /// Sends a request as the signed-in organizer whose session cookie is given.
    pub async fn send(
        state: AppState,
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
        let response = api::router(state).oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        (
            status,
            serde_json::from_slice(&bytes).unwrap_or(Value::Null),
        )
    }

    /// Stores an active member and returns the id.
    pub async fn member(pool: &PgPool, first_name: &str, language: &str) -> i64 {
        sqlx::query_scalar(
            "INSERT INTO members (first_name, last_name, mail, language) VALUES ($1, 'Tester', $1 || '@example.org', $2) RETURNING id",
        )
        .bind(first_name)
        .bind(language)
        .fetch_one(pool)
        .await
        .unwrap()
    }

    /// Stores a gathering whose days lie the given number of days from today and returns the id.
    pub async fn gathering(pool: &PgPool, title: &str, first: i32, last: i32) -> i64 {
        sqlx::query_scalar(
            "INSERT INTO gatherings (title, location, first_day, last_day) VALUES ($1, 'Kloster Drübeck', CURRENT_DATE + $2, CURRENT_DATE + $3) RETURNING id",
        )
        .bind(title)
        .bind(first)
        .bind(last)
        .fetch_one(pool)
        .await
        .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};
    use sqlx::PgPool;

    use super::testing::{gathering, member, send};
    use super::*;
    use crate::access::testing::signed_in_organizer;
    use crate::api;
    use crate::database::testing::fresh_database;

    async fn invitations(pool: &PgPool, gathering_id: i64) -> Vec<String> {
        sqlx::query_scalar(
            "SELECT members.first_name FROM invitations JOIN members ON members.id = invitations.member_id WHERE gathering_id = $1 ORDER BY members.first_name",
        )
        .bind(gathering_id)
        .fetch_all(pool)
        .await
        .unwrap()
    }

    async fn mails(pool: &PgPool) -> i64 {
        sqlx::query_scalar("SELECT count(*) FROM mail_outbox WHERE invitation_id IS NOT NULL")
            .fetch_one(pool)
            .await
            .unwrap()
    }

    async fn invite(
        pool: &PgPool,
        cookie: &str,
        gathering_id: i64,
        body: Value,
    ) -> (StatusCode, Value) {
        send(
            api::testing::state(pool.clone()),
            cookie,
            "POST",
            &format!("/api/gatherings/{gathering_id}/invitations"),
            &body,
        )
        .await
    }

    #[tokio::test]
    async fn invite_everyone() {
        let database = fresh_database().await;
        let pool = database.pool();
        let (_, cookie) = signed_in_organizer(&pool).await;
        member(&pool, "Clara", "de").await;
        member(&pool, "Johannes", "en").await;
        sqlx::query("INSERT INTO members (first_name, last_name, mail, language, active) VALUES ('Robert', 'Tester', 'robert@example.org', 'de', false)")
            .execute(&pool)
            .await
            .unwrap();
        let weekend = gathering(&pool, "Probenwochenende", 10, 12).await;
        let (status, body) = invite(&pool, &cookie, weekend, json!({})).await;
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(body, json!({ "count": 3 }));
        assert_eq!(
            invitations(&pool, weekend).await,
            vec!["Clara", "Johannes", "Root"]
        );
        assert_eq!(mails(&pool).await, 3);
    }

    #[tokio::test]
    async fn invite_a_subset() {
        let database = fresh_database().await;
        let pool = database.pool();
        let (_, cookie) = signed_in_organizer(&pool).await;
        let clara = member(&pool, "Clara", "de").await;
        let johannes = member(&pool, "Johannes", "en").await;
        member(&pool, "Robert", "de").await;
        let weekend = gathering(&pool, "Probenwochenende", 10, 12).await;
        let (status, body) = invite(
            &pool,
            &cookie,
            weekend,
            json!({ "memberIds": [clara, johannes] }),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(body, json!({ "count": 2 }));
        assert_eq!(invitations(&pool, weekend).await, vec!["Clara", "Johannes"]);
        assert_eq!(mails(&pool).await, 2);
    }

    #[tokio::test]
    async fn member_added_later() {
        let database = fresh_database().await;
        let pool = database.pool();
        let (_, cookie) = signed_in_organizer(&pool).await;
        member(&pool, "Clara", "de").await;
        let weekend = gathering(&pool, "Probenwochenende", 10, 12).await;
        invite(&pool, &cookie, weekend, json!({})).await;
        member(&pool, "Johannes", "en").await;
        let (_, body) = invite(&pool, &cookie, weekend, json!({})).await;
        assert_eq!(body, json!({ "count": 1 }));
        assert_eq!(
            invitations(&pool, weekend).await,
            vec!["Clara", "Johannes", "Root"]
        );
        assert_eq!(mails(&pool).await, 3);
    }

    async fn invitation_secret(pool: &PgPool) -> String {
        let text: String = sqlx::query_scalar("SELECT text_body FROM mail_outbox WHERE invitation_id IS NOT NULL ORDER BY id DESC LIMIT 1")
            .fetch_one(pool)
            .await
            .unwrap();
        let start = text.find("/invitation#").unwrap() + "/invitation#".len();
        text[start..start + 64].to_owned()
    }

    async fn opens(pool: &PgPool, secret: &str) -> bool {
        let mut connection = pool.acquire().await.unwrap();
        resolve(
            &mut connection,
            secret,
            time::OffsetDateTime::now_utc().date(),
        )
        .await
        .unwrap()
        .is_some()
    }

    async fn one_invitation(pool: &PgPool, cookie: &str) -> (i64, Value) {
        let clara = member(pool, "Clara", "de").await;
        let weekend = gathering(pool, "Probenwochenende", 10, 12).await;
        invite(pool, cookie, weekend, json!({ "memberIds": [clara] })).await;
        let (_, list) = send(
            api::testing::state(pool.clone()),
            cookie,
            "GET",
            &format!("/api/gatherings/{weekend}/invitations"),
            &Value::Null,
        )
        .await;
        (weekend, list[0].clone())
    }

    #[tokio::test]
    async fn resend() {
        let database = fresh_database().await;
        let pool = database.pool();
        let (_, cookie) = signed_in_organizer(&pool).await;
        let (_, invitation) = one_invitation(&pool, &cookie).await;
        assert_eq!(invitation["firstName"], "Clara");
        assert_eq!(invitation["linkActive"], true);
        assert_eq!(invitation["mail"], "waiting");
        let old = invitation_secret(&pool).await;
        let (status, _) = send(
            api::testing::state(pool.clone()),
            &cookie,
            "POST",
            &format!("/api/invitations/{}/resend", invitation["id"]),
            &Value::Null,
        )
        .await;
        assert_eq!(status, StatusCode::NO_CONTENT);
        let new = invitation_secret(&pool).await;
        assert_ne!(old, new);
        assert_eq!(mails(&pool).await, 2);
        assert!(opens(&pool, &new).await);
    }

    #[tokio::test]
    async fn old_link_after_resend() {
        let database = fresh_database().await;
        let pool = database.pool();
        let (_, cookie) = signed_in_organizer(&pool).await;
        let (_, invitation) = one_invitation(&pool, &cookie).await;
        let old = invitation_secret(&pool).await;
        send(
            api::testing::state(pool.clone()),
            &cookie,
            "POST",
            &format!("/api/invitations/{}/resend", invitation["id"]),
            &Value::Null,
        )
        .await;
        assert!(!opens(&pool, &old).await);
    }

    #[tokio::test]
    async fn revoke_without_resend() {
        let database = fresh_database().await;
        let pool = database.pool();
        let (_, cookie) = signed_in_organizer(&pool).await;
        let (weekend, invitation) = one_invitation(&pool, &cookie).await;
        let secret = invitation_secret(&pool).await;
        let (status, _) = send(
            api::testing::state(pool.clone()),
            &cookie,
            "POST",
            &format!("/api/invitations/{}/revocation", invitation["id"]),
            &Value::Null,
        )
        .await;
        assert_eq!(status, StatusCode::NO_CONTENT);
        assert!(!opens(&pool, &secret).await);
        assert_eq!(mails(&pool).await, 1);
        let (_, list) = send(
            api::testing::state(pool.clone()),
            &cookie,
            "GET",
            &format!("/api/gatherings/{weekend}/invitations"),
            &Value::Null,
        )
        .await;
        assert_eq!(list[0]["linkActive"], false);
    }

    #[tokio::test]
    async fn no_resend_after_the_gathering() {
        let database = fresh_database().await;
        let pool = database.pool();
        let (_, cookie) = signed_in_organizer(&pool).await;
        let (weekend, invitation) = one_invitation(&pool, &cookie).await;
        sqlx::query("UPDATE gatherings SET first_day = CURRENT_DATE - 3, last_day = CURRENT_DATE - 1 WHERE id = $1")
            .bind(weekend)
            .execute(&pool)
            .await
            .unwrap();
        let (status, body) = send(
            api::testing::state(pool.clone()),
            &cookie,
            "POST",
            &format!("/api/invitations/{}/resend", invitation["id"]),
            &Value::Null,
        )
        .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            body,
            json!({ "errors": [{ "field": "gathering", "code": "ended" }] })
        );
        assert_eq!(mails(&pool).await, 1);
    }

    #[tokio::test]
    async fn cancel_with_mail_to_every_invited_member() {
        let database = fresh_database().await;
        let pool = database.pool();
        let (_, cookie) = signed_in_organizer(&pool).await;
        member(&pool, "Johannes", "en").await;
        let weekend = gathering(&pool, "Probenwochenende", 10, 12).await;
        invite(&pool, &cookie, weekend, json!({})).await;
        let (status, _) = send(
            api::testing::state(pool.clone()),
            &cookie,
            "POST",
            &format!("/api/gatherings/{weekend}/cancellation"),
            &Value::Null,
        )
        .await;
        assert_eq!(status, StatusCode::NO_CONTENT);
        let cancellations: Vec<(String, String)> = sqlx::query_as(
            "SELECT recipient, subject FROM mail_outbox WHERE subject NOT LIKE 'Invitation%' AND subject NOT LIKE 'Einladung%' ORDER BY recipient",
        )
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(
            cancellations,
            vec![
                (
                    "Johannes@example.org".to_owned(),
                    "Cancelled: Probenwochenende".to_owned()
                ),
                (
                    "root@example.org".to_owned(),
                    "Cancelled: Probenwochenende".to_owned()
                ),
            ]
        );
        let (status, body) = send(
            api::testing::state(pool.clone()),
            &cookie,
            "POST",
            &format!("/api/gatherings/{weekend}/cancellation"),
            &Value::Null,
        )
        .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            body,
            json!({ "errors": [{ "field": "gathering", "code": "cancelled" }] })
        );
    }

    #[test]
    fn cancellation_mail_in_german() {
        let (member, gathering) = sample();
        let mail = cancellation_mail(&member, &gathering, 11);
        assert_eq!(mail.subject, "Abgesagt: Probenwochenende");
        assert!(mail.text.contains("Hallo Clara"), "{}", mail.text);
        assert!(
            mail.text.contains(
                "Probenwochenende in Kloster Drübeck, 12.–14. März 2027, findet nicht statt."
            ),
            "{}",
            mail.text
        );
    }

    #[tokio::test]
    async fn cancelled_gathering_takes_no_invitations() {
        let database = fresh_database().await;
        let pool = database.pool();
        let (_, cookie) = signed_in_organizer(&pool).await;
        let weekend = gathering(&pool, "Probenwochenende", 10, 12).await;
        send(
            api::testing::state(pool.clone()),
            &cookie,
            "POST",
            &format!("/api/gatherings/{weekend}/cancellation"),
            &Value::Null,
        )
        .await;
        let (status, body) = invite(&pool, &cookie, weekend, json!({})).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            body,
            json!({ "errors": [{ "field": "gathering", "code": "cancelled" }] })
        );
        assert!(invitations(&pool, weekend).await.is_empty());
    }

    #[tokio::test]
    async fn ended_gathering() {
        let database = fresh_database().await;
        let pool = database.pool();
        let (_, cookie) = signed_in_organizer(&pool).await;
        let past = gathering(&pool, "Herbstcamp", -10, -8).await;
        let (status, body) = invite(&pool, &cookie, past, json!({})).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            body,
            json!({ "errors": [{ "field": "gathering", "code": "ended" }] })
        );
        assert!(invitations(&pool, past).await.is_empty());
    }

    fn sample() -> (Member, Gathering) {
        let member = Member {
            id: 7,
            first_name: "Clara".to_owned(),
            last_name: "Schumann".to_owned(),
            mail: "clara@example.org".to_owned(),
            language: "de".to_owned(),
            active: true,
            organizer: false,
        };
        let gathering = Gathering {
            id: 3,
            title: "Probenwochenende".to_owned(),
            location: "Kloster Drübeck".to_owned(),
            description: Some("Wir proben das Programm\\nfür April.".to_owned()),
            first_day: time::macros::date!(2027 - 03 - 12),
            last_day: time::macros::date!(2027 - 03 - 14),
            cancelled: false,
            ended: false,
        };
        (member, gathering)
    }

    #[test]
    fn mail_content() {
        let (member, gathering) = sample();
        let link = "https://adunata.example.org/invitation#abc";
        let legal = LegalLinks {
            privacy: Some("https://choir.example.org/privacy".to_owned()),
            imprint: Some("https://choir.example.org/imprint".to_owned()),
        };
        let mail = invitation_mail(&member, &gathering, link, &legal, 11);
        assert_eq!(mail.subject, "Einladung: Probenwochenende");
        for expected in [
            "Hallo Clara",
            "Probenwochenende",
            "Kloster Drübeck",
            "12.–14. März 2027",
            link,
            "Datenschutzerklärung: https://choir.example.org/privacy",
            "Impressum: https://choir.example.org/imprint",
        ] {
            assert!(
                mail.text.contains(expected),
                "text lacks {expected}: {}",
                mail.text
            );
        }
        assert!(mail.html.contains(&format!("href=\"{link}\"")));
        assert!(
            mail.html
                .contains("href=\"https://choir.example.org/imprint\"")
        );
        assert_eq!((mail.member_id, mail.invitation_id), (Some(7), Some(11)));
    }

    #[test]
    fn mail_without_legal_links() {
        let (member, gathering) = sample();
        let mail = invitation_mail(
            &member,
            &gathering,
            "https://adunata.example.org/invitation#abc",
            &LegalLinks::default(),
            11,
        );
        assert!(!mail.text.contains("Datenschutz"), "{}", mail.text);
        assert!(!mail.html.contains("<hr>"));
    }

    #[test]
    fn title_with_markup() {
        let (member, mut gathering) = sample();
        gathering.title = "<b>Probe</b>".to_owned();
        let mail = invitation_mail(
            &member,
            &gathering,
            "https://adunata.example.org/invitation#abc",
            &LegalLinks::default(),
            11,
        );
        assert!(
            mail.html.contains("&lt;b&gt;Probe&lt;/b&gt;"),
            "{}",
            mail.html
        );
        assert!(!mail.html.contains("<b>"), "{}", mail.html);
    }
}
