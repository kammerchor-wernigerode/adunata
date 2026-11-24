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
        .routes(routes!(remind))
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

/// Stores the hash of a new secret in place of the old one, so the old link stops working in the same transaction, and queues a mail with the new link. It locks the gathering before the invitation, in the order of a reminder, so the two never deadlock.
async fn resend_invitation(state: &AppState, id: i64) -> Result<Outcome<()>, sqlx::Error> {
    let mut transaction = state.pool.begin().await?;
    let gathering_id: Option<i64> =
        sqlx::query_scalar("SELECT gathering_id FROM invitations WHERE id = $1")
            .bind(id)
            .fetch_optional(&mut *transaction)
            .await?;
    let Some(gathering_id) = gathering_id else {
        return Ok(Outcome::NotFound);
    };
    let gathering = match open_gathering(&mut transaction, gathering_id, state).await? {
        Ok(gathering) => gathering,
        Err(Outcome::Closed(code)) => return Ok(Outcome::Closed(code)),
        Err(_) => return Ok(Outcome::NotFound),
    };
    let member_id: Option<i64> =
        sqlx::query_scalar("SELECT member_id FROM invitations WHERE id = $1 FOR UPDATE")
            .bind(id)
            .fetch_optional(&mut *transaction)
            .await?;
    let Some(member_id) = member_id else {
        return Ok(Outcome::NotFound);
    };
    let Some(member) = members::find(&mut transaction, member_id).await? else {
        return Ok(Outcome::NotFound);
    };
    rotate_and_enqueue(
        &mut transaction,
        state,
        id,
        &member,
        &gathering,
        &INVITATION,
    )
    .await?;
    transaction.commit().await?;
    Ok(Outcome::Done(()))
}

/// Stores the hash of a new secret in place of the old one and queues a mail with the new link.
async fn rotate_and_enqueue(
    connection: &mut PgConnection,
    state: &AppState,
    invitation_id: i64,
    member: &Member,
    gathering: &Gathering,
    texts: &MailTexts,
) -> Result<(), sqlx::Error> {
    let secret = secret::generate();
    sqlx::query("UPDATE invitations SET secret_hash = $2 WHERE id = $1")
        .bind(invitation_id)
        .bind(secret::hash(&secret))
        .execute(&mut *connection)
        .await?;
    let link = personal_link(&state.public_url, &secret);
    mail::enqueue(
        connection,
        &invitation_mail(
            member,
            gathering,
            &link,
            &state.legal_links,
            invitation_id,
            texts,
        ),
    )
    .await
}

#[utoipa::path(
    post,
    path = "/api/gatherings/{id}/reminders",
    params(("id" = i64, Path, description = "Id of the gathering")),
    responses(
        (status = 201, description = "A new link created and a reminder queued for every pending member with an active link", body = Invited),
        (status = 401, description = "No valid organizer session"),
        (status = 404, description = "No such gathering"),
        (status = 422, description = "The gathering is cancelled, has ended or has begun, or its deadline for responses has passed", body = crate::validation::ValidationErrors),
    )
)]
async fn remind(_: Organizer, State(state): State<AppState>, Path(id): Path<i64>) -> Response {
    match remind_pending(&state, id).await {
        Ok(Outcome::Done(count)) => (StatusCode::CREATED, Json(Invited { count })).into_response(),
        Ok(Outcome::NotFound) => StatusCode::NOT_FOUND.into_response(),
        Ok(Outcome::Closed(code)) => invalid(vec![field_error("gathering", code)]),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// Gives every pending invitation with an active link a new secret and queues a reminder, in one transaction.
async fn remind_pending(
    state: &AppState,
    gathering_id: i64,
) -> Result<Outcome<usize>, sqlx::Error> {
    let mut transaction = state.pool.begin().await?;
    let gathering = match open_gathering(&mut transaction, gathering_id, state).await? {
        Ok(gathering) => gathering,
        Err(outcome) => return Ok(outcome),
    };
    if gathering.begun {
        return Ok(Outcome::Closed(ErrorCode::Begun));
    }
    if gathering.deadline_passed {
        return Ok(Outcome::Closed(ErrorCode::DeadlinePassed));
    }
    let pending: Vec<(i64, i64)> = sqlx::query_as(
        "SELECT id, member_id FROM invitations WHERE gathering_id = $1 AND secret_hash IS NOT NULL AND NOT EXISTS (SELECT 1 FROM responses WHERE responses.invitation_id = invitations.id) ORDER BY id FOR UPDATE",
    )
    .bind(gathering_id)
    .fetch_all(&mut *transaction)
    .await?;
    let mut count = 0;
    for (invitation_id, member_id) in pending {
        if let Some(member) = members::find(&mut transaction, member_id).await? {
            rotate_and_enqueue(
                &mut transaction,
                state,
                invitation_id,
                &member,
                &gathering,
                &REMINDER,
            )
            .await?;
            count += 1;
        }
    }
    transaction.commit().await?;
    Ok(Outcome::Done(count))
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

/// Finds and locks a gathering that still takes invitations, or tells why it does not.
async fn open_gathering(
    connection: &mut PgConnection,
    id: i64,
    state: &AppState,
) -> Result<Result<Gathering, Outcome<usize>>, sqlx::Error> {
    let Some(gathering) =
        gatherings::find_for_update(connection, id, state.time_zone.today()).await?
    else {
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
            &INVITATION,
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

/// The message ids of the lines that tell an invitation from a reminder.
struct MailTexts {
    subject: &'static str,
    details: &'static str,
}

const INVITATION: MailTexts = MailTexts {
    subject: "invitation-mail-subject",
    details: "invitation-mail-details",
};

const REMINDER: MailTexts = MailTexts {
    subject: "reminder-mail-subject",
    details: "reminder-mail-details",
};

fn invitation_mail(
    member: &Member,
    gathering: &Gathering,
    link: &str,
    legal_links: &LegalLinks,
    invitation_id: i64,
    texts: &MailTexts,
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
    let subject = localizer.message(language, texts.subject, &args);
    let greeting = localizer.message(language, "invitation-mail-greeting", &args);
    let details = localizer.message(language, texts.details, &args);
    let answer = localizer.message(language, "invitation-mail-answer", &args);
    let mut text = format!("{greeting}\n\n{details}\n\n");
    let mut html = format!(
        "<p>{}</p><p>{}</p>",
        mail::escape_html(&greeting),
        mail::escape_html(&details)
    );
    if let Some(deadline) = deadline_line(gathering, language) {
        write!(text, "{deadline}\n\n").expect("writing to a string cannot fail");
        write!(html, "<p>{}</p>", mail::escape_html(&deadline))
            .expect("writing to a string cannot fail");
    }
    write!(text, "{answer}\n{link}\n").expect("writing to a string cannot fail");
    write!(
        html,
        "<p>{}<br><a href=\"{link}\">{link}</a></p>",
        mail::escape_html(&answer)
    )
    .expect("writing to a string cannot fail");
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

/// The line that names the last day to answer, or that it has passed, or `None` without a deadline.
fn deadline_line(gathering: &Gathering, language: &str) -> Option<String> {
    let day = gathering.last_day_to_answer?;
    let localizer = localization::localizer();
    let mut args = FluentArgs::new();
    args.set("day", localizer.date_span(language, day, day));
    let id = if gathering.deadline_passed {
        "invitation-mail-deadline-passed"
    } else {
        "invitation-mail-deadline"
    };
    Some(localizer.message(language, id, &args))
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
            "INSERT INTO gatherings (title, location, first_day, last_day) VALUES ($1, 'Kloster Drübeck', $2, $3) RETURNING id",
        )
        .bind(title)
        .bind(crate::api::testing::day(first))
        .bind(crate::api::testing::day(last))
        .fetch_one(pool)
        .await
        .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancellation_mail_in_german() {
        let (member, gathering) = sample();
        let mail = cancellation_mail(&member, &gathering, 11);
        assert_eq!(
            mail.subject,
            "Abgesagt: Probenwochenende, 12.–14. März 2027"
        );
        assert!(mail.text.contains("Hallo Clara"), "{}", mail.text);
        assert!(
            mail.text.contains(
                "Probenwochenende in Kloster Drübeck, 12.–14. März 2027, findet nicht statt."
            ),
            "{}",
            mail.text
        );
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
            response_deadline_days: None,
            last_day_to_answer: None,
            deadline_passed: false,
            cancelled: false,
            begun: false,
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
        let mail = invitation_mail(&member, &gathering, link, &legal, 11, &INVITATION);
        assert_eq!(
            mail.subject,
            "Einladung: Probenwochenende, 12.–14. März 2027"
        );
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
    fn reminder_subjects() {
        let (mut member, gathering) = sample();
        let link = "https://adunata.example.org/invitation#abc";
        let mail = invitation_mail(
            &member,
            &gathering,
            link,
            &LegalLinks::default(),
            11,
            &REMINDER,
        );
        assert_eq!(
            mail.subject,
            "Erinnerung: Probenwochenende, 12.–14. März 2027"
        );
        member.language = "en".to_owned();
        let mail = invitation_mail(
            &member,
            &gathering,
            link,
            &LegalLinks::default(),
            11,
            &REMINDER,
        );
        assert!(
            mail.subject.starts_with("Reminder: Probenwochenende, "),
            "{}",
            mail.subject
        );
    }

    #[test]
    fn reminder_content() {
        let (member, gathering) = sample();
        let link = "https://adunata.example.org/invitation#abc";
        let legal = LegalLinks {
            privacy: Some("https://choir.example.org/privacy".to_owned()),
            imprint: Some("https://choir.example.org/imprint".to_owned()),
        };
        let mail = invitation_mail(&member, &gathering, link, &legal, 11, &REMINDER);
        for expected in [
            "noch keine Antwort",
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
            &INVITATION,
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
            &INVITATION,
        );
        assert!(
            mail.html.contains("&lt;b&gt;Probe&lt;/b&gt;"),
            "{}",
            mail.html
        );
        assert!(!mail.html.contains("<b>"), "{}", mail.html);
    }

    fn with_deadline(mut gathering: Gathering, passed: bool) -> Gathering {
        gathering.response_deadline_days = Some(7);
        gathering.last_day_to_answer = Some(time::macros::date!(2027 - 03 - 06));
        gathering.deadline_passed = passed;
        gathering
    }

    #[test]
    fn invitation_with_a_deadline() {
        let (member, gathering) = sample();
        let mail = invitation_mail(
            &member,
            &with_deadline(gathering, false),
            "https://adunata.example.org/invitation#abc",
            &LegalLinks::default(),
            11,
            &INVITATION,
        );
        assert!(mail.text.contains("6. März 2027"), "{}", mail.text);
        assert!(mail.html.contains("6. März 2027"), "{}", mail.html);
    }

    #[test]
    fn reminder_with_a_deadline() {
        let (mut member, gathering) = sample();
        member.language = "en".to_owned();
        let mail = invitation_mail(
            &member,
            &with_deadline(gathering, false),
            "https://adunata.example.org/invitation#abc",
            &LegalLinks::default(),
            11,
            &REMINDER,
        );
        assert!(mail.text.contains("March 6, 2027"), "{}", mail.text);
    }

    #[test]
    fn invitation_after_the_deadline() {
        let (mut member, gathering) = sample();
        member.language = "en".to_owned();
        let mail = invitation_mail(
            &member,
            &with_deadline(gathering, true),
            "https://adunata.example.org/invitation#abc",
            &LegalLinks::default(),
            11,
            &INVITATION,
        );
        assert!(
            mail.text
                .contains("The deadline for responses passed on March 6, 2027. You can still decline through your personal link."),
            "{}",
            mail.text
        );
    }

    #[test]
    fn invitation_without_a_deadline() {
        let (mut member, gathering) = sample();
        member.language = "en".to_owned();
        let mail = invitation_mail(
            &member,
            &gathering,
            "https://adunata.example.org/invitation#abc",
            &LegalLinks::default(),
            11,
            &INVITATION,
        );
        assert!(!mail.text.contains("March 6"), "{}", mail.text);
        assert!(!mail.text.contains("deadline"), "{}", mail.text);
    }

    mod integration {
        use super::testing::{gathering, member, send};
        use super::*;
        use crate::access::testing::signed_in_organizer;
        use crate::api;
        use crate::database::testing::fresh_database;
        use serde_json::{Value, json};
        use sqlx::PgPool;

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
        async fn invitation_succeeds_while_the_mail_server_is_down() {
            let database = fresh_database().await;
            let pool = database.pool();
            let (_, cookie) = signed_in_organizer(&pool).await;
            member(&pool, "Clara", "de").await;
            let weekend = gathering(&pool, "Probenwochenende", 10, 12).await;
            let (status, _) = invite(&pool, &cookie, weekend, json!({})).await;
            assert_eq!(status, StatusCode::CREATED);
            let unreachable = mail::Delivery {
                transport: mail::transport(&crate::configuration::Smtp {
                    host: "127.0.0.1".to_owned(),
                    port: 1,
                    encryption: crate::configuration::Encryption::None,
                    username: None,
                    password: None,
                })
                .unwrap(),
                sender: "Adunata <adunata@example.org>".parse().unwrap(),
                max_attempts: 8,
                confine_to: None,
            };
            mail::deliver_due(&pool, &unreachable).await.unwrap();
            let waiting: i64 = sqlx::query_scalar("SELECT count(*) FROM mail_outbox WHERE sent_at IS NULL AND failed_at IS NULL AND recipient IS NOT NULL")
                    .fetch_one(&pool)
                    .await
                    .unwrap();
            assert_eq!(
                waiting, 2,
                "the mail waits in the outbox until the server is reachable again"
            );
            assert_eq!(invitations(&pool, weekend).await, vec!["Clara", "Root"]);
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
            sqlx::query("UPDATE gatherings SET first_day = $2, last_day = $3 WHERE id = $1")
                .bind(weekend)
                .bind(api::testing::day(-3))
                .bind(api::testing::day(-1))
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
            let span = localization::localizer().date_span(
                "en",
                api::testing::day(10),
                api::testing::day(12),
            );
            let subject = format!("Cancelled: Probenwochenende, {span}");
            let cancellations: Vec<(String, String)> = sqlx::query_as(
                    "SELECT recipient, subject FROM mail_outbox WHERE subject NOT LIKE 'Invitation%' AND subject NOT LIKE 'Einladung%' ORDER BY recipient",
                )
                .fetch_all(&pool)
                .await
                .unwrap();
            assert_eq!(
                cancellations,
                vec![
                    ("Johannes@example.org".to_owned(), subject.clone()),
                    ("root@example.org".to_owned(), subject),
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

        #[tokio::test]
        async fn cancellation_waits_for_an_invitation_in_progress() {
            let database = fresh_database().await;
            let pool = database.pool();
            signed_in_organizer(&pool).await;
            let clara = member(&pool, "Clara", "de").await;
            let weekend = gathering(&pool, "Probenwochenende", 10, 12).await;
            let state = api::testing::state(pool.clone());

            let mut transaction = pool.begin().await.unwrap();
            assert!(
                open_gathering(&mut transaction, weekend, &state)
                    .await
                    .unwrap()
                    .is_ok()
            );
            let cancellation = tokio::spawn({
                let state = state.clone();
                async move { cancel_gathering(&state, weekend).await }
            });
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            sqlx::query("INSERT INTO invitations (gathering_id, member_id) VALUES ($1, $2)")
                .bind(weekend)
                .bind(clara)
                .execute(&mut *transaction)
                .await
                .unwrap();
            transaction.commit().await.unwrap();
            assert!(matches!(
                cancellation.await.unwrap().unwrap(),
                Outcome::Done(())
            ));

            let cancellation_mails: i64 = sqlx::query_scalar(
                "SELECT count(*) FROM mail_outbox WHERE subject LIKE 'Abgesagt%'",
            )
            .fetch_one(&pool)
            .await
            .unwrap();
            assert_eq!(
                cancellation_mails, 1,
                "the member invited meanwhile hears of the cancellation"
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

        async fn remind(pool: &PgPool, cookie: &str, gathering_id: i64) -> (StatusCode, Value) {
            send(
                api::testing::state(pool.clone()),
                cookie,
                "POST",
                &format!("/api/gatherings/{gathering_id}/reminders"),
                &Value::Null,
            )
            .await
        }

        async fn reminded(pool: &PgPool) -> Vec<String> {
            sqlx::query_scalar("SELECT recipient FROM mail_outbox WHERE subject LIKE 'Erinnerung%' OR subject LIKE 'Reminder%' ORDER BY recipient")
                .fetch_all(pool)
                .await
                .unwrap()
        }

        async fn answer(pool: &PgPool, gathering_id: i64, first_name: &str, status: &str) {
            sqlx::query("INSERT INTO responses (invitation_id, status, beds, changed_by) SELECT invitations.id, $3, 'none', 'member' FROM invitations JOIN members ON members.id = invitations.member_id WHERE gathering_id = $1 AND members.first_name = $2")
                .bind(gathering_id)
                .bind(first_name)
                .bind(status)
                .execute(pool)
                .await
                .unwrap();
        }

        async fn secret_hash(
            pool: &PgPool,
            gathering_id: i64,
            first_name: &str,
        ) -> Option<Vec<u8>> {
            sqlx::query_scalar("SELECT invitations.secret_hash FROM invitations JOIN members ON members.id = invitations.member_id WHERE gathering_id = $1 AND members.first_name = $2")
                .bind(gathering_id)
                .bind(first_name)
                .fetch_one(pool)
                .await
                .unwrap()
        }

        #[tokio::test]
        async fn remind_pending_members() {
            let database = fresh_database().await;
            let pool = database.pool();
            let (_, cookie) = signed_in_organizer(&pool).await;
            member(&pool, "Clara", "de").await;
            member(&pool, "Johannes", "en").await;
            member(&pool, "Robert", "de").await;
            let weekend = gathering(&pool, "Probenwochenende", 10, 12).await;
            invite(&pool, &cookie, weekend, json!({})).await;
            answer(&pool, weekend, "Root", "accepted").await;
            answer(&pool, weekend, "Robert", "tentative").await;
            sqlx::query("UPDATE members SET active = false WHERE first_name = 'Johannes'")
                .execute(&pool)
                .await
                .unwrap();
            let (status, body) = remind(&pool, &cookie, weekend).await;
            assert_eq!(status, StatusCode::CREATED);
            assert_eq!(body, json!({ "count": 2 }));
            assert_eq!(
                reminded(&pool).await,
                vec!["Clara@example.org", "Johannes@example.org"]
            );
            let (_, body) = remind(&pool, &cookie, weekend).await;
            assert_eq!(
                body,
                json!({ "count": 2 }),
                "a second reminder goes out too"
            );
            assert_eq!(reminded(&pool).await.len(), 4);
        }

        #[tokio::test]
        async fn old_link_after_reminder() {
            let database = fresh_database().await;
            let pool = database.pool();
            let (_, cookie) = signed_in_organizer(&pool).await;
            let (weekend, _) = one_invitation(&pool, &cookie).await;
            answer(&pool, weekend, "Root", "accepted").await;
            let old = invitation_secret(&pool).await;
            remind(&pool, &cookie, weekend).await;
            let new = invitation_secret(&pool).await;
            assert!(!opens(&pool, &old).await);
            assert!(opens(&pool, &new).await);
        }

        #[tokio::test]
        async fn reminder_skips_revoked_and_includes_failed() {
            let database = fresh_database().await;
            let pool = database.pool();
            let (_, cookie) = signed_in_organizer(&pool).await;
            member(&pool, "Clara", "de").await;
            member(&pool, "Johannes", "en").await;
            let weekend = gathering(&pool, "Probenwochenende", 10, 12).await;
            invite(&pool, &cookie, weekend, json!({})).await;
            answer(&pool, weekend, "Root", "accepted").await;
            sqlx::query("UPDATE invitations SET secret_hash = NULL WHERE member_id = (SELECT id FROM members WHERE first_name = 'Clara')")
                .execute(&pool)
                .await
                .unwrap();
            sqlx::query(
                "UPDATE mail_outbox SET failed_at = now(), recipient = NULL, subject = NULL, text_body = NULL, html_body = NULL WHERE member_id = (SELECT id FROM members WHERE first_name = 'Johannes')",
            )
            .execute(&pool)
            .await
            .unwrap();
            let (_, body) = remind(&pool, &cookie, weekend).await;
            assert_eq!(body, json!({ "count": 1 }));
            assert_eq!(reminded(&pool).await, vec!["Johannes@example.org"]);
            assert_eq!(secret_hash(&pool, weekend, "Clara").await, None);
        }

        #[tokio::test]
        async fn nobody_to_remind() {
            let database = fresh_database().await;
            let pool = database.pool();
            let (_, cookie) = signed_in_organizer(&pool).await;
            let weekend = gathering(&pool, "Probenwochenende", 10, 12).await;
            invite(&pool, &cookie, weekend, json!({})).await;
            answer(&pool, weekend, "Root", "accepted").await;
            let (status, body) = remind(&pool, &cookie, weekend).await;
            assert_eq!(status, StatusCode::CREATED);
            assert_eq!(body, json!({ "count": 0 }));
            assert_eq!(mails(&pool).await, 1);
        }

        #[tokio::test]
        async fn no_reminder_for_a_closed_gathering() {
            let database = fresh_database().await;
            let pool = database.pool();
            let (_, cookie) = signed_in_organizer(&pool).await;
            let weekend = gathering(&pool, "Probenwochenende", 10, 12).await;
            invite(&pool, &cookie, weekend, json!({})).await;
            let before = secret_hash(&pool, weekend, "Root").await;
            sqlx::query("UPDATE gatherings SET first_day = $2, last_day = $3 WHERE id = $1")
                .bind(weekend)
                .bind(api::testing::day(-3))
                .bind(api::testing::day(-1))
                .execute(&pool)
                .await
                .unwrap();
            let (status, body) = remind(&pool, &cookie, weekend).await;
            assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
            assert_eq!(
                body,
                json!({ "errors": [{ "field": "gathering", "code": "ended" }] })
            );
            sqlx::query("UPDATE gatherings SET first_day = $2, last_day = $3, cancelled_at = now() WHERE id = $1")
                .bind(weekend)
                .bind(api::testing::day(10))
                .bind(api::testing::day(12))
                .execute(&pool)
                .await
                .unwrap();
            let (status, body) = remind(&pool, &cookie, weekend).await;
            assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
            assert_eq!(
                body,
                json!({ "errors": [{ "field": "gathering", "code": "cancelled" }] })
            );
            assert_eq!(secret_hash(&pool, weekend, "Root").await, before);
            assert_eq!(mails(&pool).await, 1);
        }

        #[tokio::test]
        async fn no_reminder_once_the_gathering_has_begun() {
            let database = fresh_database().await;
            let pool = database.pool();
            let (_, cookie) = signed_in_organizer(&pool).await;
            let weekend = gathering(&pool, "Probenwochenende", 0, 2).await;
            invite(&pool, &cookie, weekend, json!({})).await;
            let before = secret_hash(&pool, weekend, "Root").await;
            let (status, body) = remind(&pool, &cookie, weekend).await;
            assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
            assert_eq!(
                body,
                json!({ "errors": [{ "field": "gathering", "code": "begun" }] })
            );
            assert_eq!(secret_hash(&pool, weekend, "Root").await, before);
            assert_eq!(mails(&pool).await, 1);
        }

        #[tokio::test]
        async fn reminder_on_the_day_before_the_gathering() {
            let database = fresh_database().await;
            let pool = database.pool();
            let (_, cookie) = signed_in_organizer(&pool).await;
            let weekend = gathering(&pool, "Probenwochenende", 1, 2).await;
            invite(&pool, &cookie, weekend, json!({})).await;
            let (status, body) = remind(&pool, &cookie, weekend).await;
            assert_eq!(status, StatusCode::CREATED);
            assert_eq!(body, json!({ "count": 1 }));
        }

        async fn set_deadline(pool: &PgPool, gathering_id: i64, days: i32) {
            sqlx::query("UPDATE gatherings SET response_deadline_days = $2 WHERE id = $1")
                .bind(gathering_id)
                .bind(days)
                .execute(pool)
                .await
                .unwrap();
        }

        #[tokio::test]
        async fn remind_after_the_deadline() {
            let database = fresh_database().await;
            let pool = database.pool();
            let (_, cookie) = signed_in_organizer(&pool).await;
            let weekend = gathering(&pool, "Probenwochenende", 10, 12).await;
            invite(&pool, &cookie, weekend, json!({})).await;
            set_deadline(&pool, weekend, 10).await;
            let before = secret_hash(&pool, weekend, "Root").await;
            let (status, body) = remind(&pool, &cookie, weekend).await;
            assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
            assert_eq!(
                body,
                json!({ "errors": [{ "field": "gathering", "code": "deadlinePassed" }] })
            );
            assert_eq!(secret_hash(&pool, weekend, "Root").await, before);
            assert_eq!(mails(&pool).await, 1);
        }

        #[tokio::test]
        async fn remind_on_the_last_day_to_answer() {
            let database = fresh_database().await;
            let pool = database.pool();
            let (_, cookie) = signed_in_organizer(&pool).await;
            let weekend = gathering(&pool, "Probenwochenende", 10, 12).await;
            invite(&pool, &cookie, weekend, json!({})).await;
            set_deadline(&pool, weekend, 9).await;
            let (status, body) = remind(&pool, &cookie, weekend).await;
            assert_eq!(status, StatusCode::CREATED);
            assert_eq!(body, json!({ "count": 1 }));
        }

        #[tokio::test]
        async fn invite_and_resend_after_the_deadline() {
            let database = fresh_database().await;
            let pool = database.pool();
            let (_, cookie) = signed_in_organizer(&pool).await;
            let weekend = gathering(&pool, "Probenwochenende", 10, 12).await;
            set_deadline(&pool, weekend, 10).await;
            let (status, _) = invite(&pool, &cookie, weekend, json!({})).await;
            assert_eq!(status, StatusCode::CREATED);
            let (_, list) = send(
                api::testing::state(pool.clone()),
                &cookie,
                "GET",
                &format!("/api/gatherings/{weekend}/invitations"),
                &Value::Null,
            )
            .await;
            let (status, _) = send(
                api::testing::state(pool.clone()),
                &cookie,
                "POST",
                &format!("/api/invitations/{}/resend", list[0]["id"]),
                &Value::Null,
            )
            .await;
            assert_eq!(status, StatusCode::NO_CONTENT);
            assert_eq!(mails(&pool).await, 2);
        }

        #[tokio::test]
        async fn invite_and_resend_once_the_gathering_has_begun() {
            let database = fresh_database().await;
            let pool = database.pool();
            let (_, cookie) = signed_in_organizer(&pool).await;
            let weekend = gathering(&pool, "Probenwochenende", 0, 2).await;
            let (status, _) = invite(&pool, &cookie, weekend, json!({})).await;
            assert_eq!(status, StatusCode::CREATED);
            let (_, list) = send(
                api::testing::state(pool.clone()),
                &cookie,
                "GET",
                &format!("/api/gatherings/{weekend}/invitations"),
                &Value::Null,
            )
            .await;
            let (status, _) = send(
                api::testing::state(pool.clone()),
                &cookie,
                "POST",
                &format!("/api/invitations/{}/resend", list[0]["id"]),
                &Value::Null,
            )
            .await;
            assert_eq!(status, StatusCode::NO_CONTENT);
            assert_eq!(mails(&pool).await, 2);
        }
    }
}
