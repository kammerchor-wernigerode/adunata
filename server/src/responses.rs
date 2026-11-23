use axum::Json;
use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use sqlx::PgConnection;
use time::{Date, PrimitiveDateTime};
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::access::{self, Organizer};
use crate::api::AppState;
use crate::calendar;
use crate::configuration::{InstanceTimeZone, Retention, RetentionPeriod};
use crate::gatherings;
use crate::gatherings::Gathering;
use crate::invitations::{self, MailState, PersonalLink};
use crate::validation::{ErrorCode, FieldError, field_error, invalid};

time::serde::format_description!(iso_date, Date, "[year]-[month]-[day]");
time::serde::format_description!(
    wall_clock,
    PrimitiveDateTime,
    "[year]-[month]-[day]T[hour]:[minute]"
);

const MAX_BEDS: i16 = 20;
const MAX_COMMENT_LENGTH: usize = 2000;
const STEP_MINUTES: u8 = 30;

/// Whether a member comes. An invitation without a response is pending.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[serde(rename_all = "camelCase")]
#[sqlx(type_name = "text", rename_all = "lowercase")]
pub enum ResponseStatus {
    Accepted,
    Tentative,
    Declined,
}

/// Whether a member who comes needs beds, offers beds or needs none.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[serde(rename_all = "camelCase")]
#[sqlx(type_name = "text", rename_all = "lowercase")]
pub enum Beds {
    Need,
    Offer,
    None,
}

/// Who changed a response last.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[serde(rename_all = "camelCase")]
#[sqlx(type_name = "text", rename_all = "lowercase")]
pub enum ChangedBy {
    Member,
    Organizer,
}

/// The answer to an invitation. Times are wall-clock times of the instance without an offset, such as 2027-03-14T18:00.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ResponseValues {
    pub status: ResponseStatus,
    #[serde(default, with = "wall_clock::option")]
    #[schema(value_type = Option<String>, example = "2027-03-14T18:00")]
    pub arrival: Option<PrimitiveDateTime>,
    #[serde(default, with = "wall_clock::option")]
    #[schema(value_type = Option<String>, example = "2027-03-16T15:00")]
    pub departure: Option<PrimitiveDateTime>,
    #[serde(default)]
    pub beds: Option<Beds>,
    #[serde(default)]
    #[schema(minimum = 1, maximum = 20)]
    pub bed_count: Option<i16>,
    #[serde(default)]
    #[schema(max_length = 2000)]
    pub comment: Option<String>,
}

impl ResponseValues {
    /// Clears what a decline makes meaningless and trims the comment.
    fn normalized(&self) -> Self {
        let declined = self.status == ResponseStatus::Declined;
        let counted = matches!(self.beds, Some(Beds::Need | Beds::Offer));
        Self {
            status: self.status,
            arrival: if declined { None } else { self.arrival },
            departure: if declined { None } else { self.departure },
            beds: if declined { None } else { self.beds },
            bed_count: if declined || !counted {
                None
            } else {
                self.bed_count
            },
            comment: self
                .comment
                .as_deref()
                .map(str::trim)
                .filter(|comment| !comment.is_empty())
                .map(str::to_owned),
        }
    }

    fn validate(&self, gathering: &Gathering) -> Vec<FieldError> {
        let mut errors = Vec::new();
        if self.status != ResponseStatus::Declined {
            for (field, time) in [("arrival", self.arrival), ("departure", self.departure)] {
                if let Some(time) = time
                    && !(on_a_step(time) && within(time, gathering))
                {
                    errors.push(field_error(field, ErrorCode::Invalid));
                }
            }
            if let (Some(arrival), Some(departure)) = (self.arrival, self.departure)
                && departure < arrival
                && !errors.iter().any(|error| error.field == "departure")
            {
                errors.push(field_error("departure", ErrorCode::BeforeArrival));
            }
            match (self.beds, self.bed_count) {
                (None, _) => errors.push(field_error("beds", ErrorCode::Required)),
                (Some(Beds::Need | Beds::Offer), None) => {
                    errors.push(field_error("bedCount", ErrorCode::Required));
                }
                (Some(Beds::Need | Beds::Offer), Some(count))
                    if !(1..=MAX_BEDS).contains(&count) =>
                {
                    errors.push(field_error("bedCount", ErrorCode::TooLarge));
                }
                _ => {}
            }
        }
        if self
            .comment
            .as_deref()
            .is_some_and(|comment| comment.trim().chars().count() > MAX_COMMENT_LENGTH)
        {
            errors.push(field_error("comment", ErrorCode::TooLong));
        }
        errors
    }
}

fn on_a_step(time: PrimitiveDateTime) -> bool {
    time.minute().is_multiple_of(STEP_MINUTES) && time.second() == 0 && time.nanosecond() == 0
}

fn within(time: PrimitiveDateTime, gathering: &Gathering) -> bool {
    (gathering.first_day..=gathering.last_day).contains(&time.date())
}

/// Applies the retention periods once per interval: clears comments, deletes the invitations and responses of long past gatherings and lets `access` delete ended sessions and used sign-in links.
pub async fn run_retention(
    pool: sqlx::PgPool,
    retention: Retention,
    time_zone: InstanceTimeZone,
    interval: std::time::Duration,
) {
    loop {
        if let Err(error) = expire(&pool, retention, time_zone.today()).await {
            tracing::error!(%error, "expired personal data could not be deleted");
        }
        tokio::time::sleep(interval).await;
    }
}

/// The first day that a period of retention still keeps, counted from the last day of a gathering.
fn kept_since(period: RetentionPeriod, today: Date) -> Option<Date> {
    match period {
        RetentionPeriod::Never => None,
        RetentionPeriod::Days(days) => Some(today - time::Duration::days(i64::from(days))),
    }
}

async fn expire(pool: &sqlx::PgPool, retention: Retention, today: Date) -> Result<(), sqlx::Error> {
    let mut transaction = pool.begin().await?;
    if let Some(since) = kept_since(retention.comments, today) {
        sqlx::query(
            "UPDATE responses SET comment = NULL FROM invitations JOIN gatherings ON gatherings.id = invitations.gathering_id WHERE invitations.id = responses.invitation_id AND responses.comment IS NOT NULL AND gatherings.last_day < $1",
        )
        .bind(since)
        .execute(&mut *transaction)
        .await?;
    }
    if let Some(since) = kept_since(retention.responses, today) {
        invitations::delete_of_gatherings_ended_before(&mut transaction, since).await?;
    }
    access::delete_expired(&mut transaction).await?;
    transaction.commit().await
}

/// Stores a response for an invitation in place of the earlier one.
async fn store(
    connection: &mut PgConnection,
    invitation_id: i64,
    values: &ResponseValues,
    changed_by: ChangedBy,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO responses (invitation_id, status, arrival, departure, beds, bed_count, comment, changed_by, changed_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, now()) ON CONFLICT (invitation_id) DO UPDATE SET status = $2, arrival = $3, departure = $4, beds = $5, bed_count = $6, comment = $7, changed_by = $8, changed_at = now()",
    )
    .bind(invitation_id)
    .bind(values.status)
    .bind(values.arrival)
    .bind(values.departure)
    .bind(values.beds)
    .bind(values.bed_count)
    .bind(&values.comment)
    .bind(changed_by)
    .execute(connection)
    .await
    .map(|_| ())
}

/// The stored response of an invitation, if any.
async fn stored(
    connection: &mut PgConnection,
    invitation_id: i64,
) -> Result<Option<ResponseValues>, sqlx::Error> {
    sqlx::query_as(
        "SELECT status, arrival, departure, beds, bed_count, comment FROM responses WHERE invitation_id = $1",
    )
    .bind(invitation_id)
    .fetch_optional(connection)
    .await
}

/// Whether a member can still answer through the personal link.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum LinkState {
    /// The gathering has not begun, so the member answers or changes the answer.
    Open,
    /// The first day has begun. The member sees the answer without a form.
    Begun,
    /// The last day has passed. The member sees the gathering and the own answer only.
    Ended,
    /// Organizers cancelled the gathering. The member sees that and no form.
    Cancelled,
}

/// What a member sees of the gathering on the response page.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct InvitedGathering {
    pub title: String,
    pub location: String,
    pub description: Option<String>,
    #[serde(with = "iso_date")]
    #[schema(value_type = String, format = Date)]
    pub first_day: Date,
    #[serde(with = "iso_date")]
    #[schema(value_type = String, format = Date)]
    pub last_day: Date,
}

/// The response page of one invitation as its member sees it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct InvitationPage {
    pub first_name: String,
    pub gathering: InvitedGathering,
    pub state: LinkState,
    /// The member's stored response, which the form shows again.
    pub response: Option<ResponseValues>,
    /// Who comes. Absent once the gathering has ended or when it is cancelled.
    pub attendees: Option<Attendees>,
    /// Whether the page offers the calendar file of the gathering.
    pub calendar: bool,
}

/// A member who accepted or accepted tentatively, as other invited members see them.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Attendee {
    pub name: String,
    pub status: ResponseStatus,
}

/// What invited members see of each other. Names and status for those who come, counts for everyone else, and never a period, beds, comment or mail address.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Attendees {
    pub coming: Vec<Attendee>,
    pub declined: i64,
    pub pending: i64,
}

async fn attendees(
    connection: &mut PgConnection,
    gathering_id: i64,
) -> Result<Attendees, sqlx::Error> {
    let coming = sqlx::query_as(
        "SELECT members.first_name || ' ' || members.last_name AS name, responses.status FROM invitations JOIN members ON members.id = invitations.member_id JOIN responses ON responses.invitation_id = invitations.id WHERE invitations.gathering_id = $1 AND responses.status IN ('accepted', 'tentative') ORDER BY responses.status, lower(members.first_name), lower(members.last_name)",
    )
    .bind(gathering_id)
    .fetch_all(&mut *connection)
    .await?;
    let (declined, pending): (i64, i64) = sqlx::query_as(
        "SELECT count(*) FILTER (WHERE responses.status = 'declined'), count(*) FILTER (WHERE responses.invitation_id IS NULL) FROM invitations LEFT JOIN responses ON responses.invitation_id = invitations.id WHERE invitations.gathering_id = $1",
    )
    .bind(gathering_id)
    .fetch_one(connection)
    .await?;
    Ok(Attendees {
        coming,
        declined,
        pending,
    })
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(invitation_page))
        .routes(routes!(calendar_file))
        .routes(routes!(answer))
        .routes(routes!(answer_for_member))
        .routes(routes!(overview))
}

/// Where an invitation stands. Without a response it is pending.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum InvitationStatus {
    Pending,
    Accepted,
    Tentative,
    Declined,
}

/// One invitation in the overview with its response.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OverviewRow {
    pub invitation_id: i64,
    pub first_name: String,
    pub last_name: String,
    pub status: InvitationStatus,
    #[serde(with = "wall_clock::option")]
    #[schema(value_type = Option<String>, example = "2027-03-14T18:00")]
    pub arrival: Option<PrimitiveDateTime>,
    #[serde(with = "wall_clock::option")]
    #[schema(value_type = Option<String>, example = "2027-03-16T15:00")]
    pub departure: Option<PrimitiveDateTime>,
    pub beds: Option<Beds>,
    pub bed_count: Option<i16>,
    pub comment: Option<String>,
    pub changed_by: Option<ChangedBy>,
    /// When the response changed last, as wall-clock time of the instance.
    #[serde(with = "wall_clock::option")]
    #[schema(value_type = Option<String>, example = "2027-02-20T09:30")]
    pub changed_at: Option<PrimitiveDateTime>,
    /// Whether the stated period lies partly or wholly outside the days of the gathering, after organizers changed them.
    pub outside_span: bool,
    pub link_active: bool,
    pub mail: Option<MailState>,
}

/// Counts per status and the beds that attending members need and offer.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Totals {
    pub accepted: i64,
    pub tentative: i64,
    pub declined: i64,
    pub pending: i64,
    pub beds_needed: i64,
    pub beds_offered: i64,
}

/// Everything organizers plan a gathering with.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Overview {
    pub rows: Vec<OverviewRow>,
    pub totals: Totals,
}

#[derive(sqlx::FromRow)]
struct StoredResponse {
    invitation_id: i64,
    status: ResponseStatus,
    arrival: Option<PrimitiveDateTime>,
    departure: Option<PrimitiveDateTime>,
    beds: Option<Beds>,
    bed_count: Option<i16>,
    comment: Option<String>,
    changed_by: ChangedBy,
    changed_at: time::OffsetDateTime,
}

#[utoipa::path(
    get,
    path = "/api/gatherings/{id}/overview",
    params(("id" = i64, Path, description = "Id of the gathering")),
    responses(
        (status = 200, description = "Every invitation with its response and the totals", body = Overview),
        (status = 401, description = "No valid organizer session"),
        (status = 404, description = "No such gathering"),
    )
)]
async fn overview(_: Organizer, State(state): State<AppState>, Path(id): Path<i64>) -> Response {
    let Ok(mut connection) = state.pool.acquire().await else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    match build_overview(&mut connection, &state, id).await {
        Ok(Some(overview)) => Json(overview).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn build_overview(
    connection: &mut PgConnection,
    state: &AppState,
    gathering_id: i64,
) -> Result<Option<Overview>, sqlx::Error> {
    let Some(gathering) =
        gatherings::find(connection, gathering_id, state.time_zone.today()).await?
    else {
        return Ok(None);
    };
    let invitations = invitations::of_gathering(connection, gathering_id).await?;
    let responses: std::collections::HashMap<i64, StoredResponse> = sqlx::query_as::<_, StoredResponse>(
        "SELECT responses.invitation_id, status, arrival, departure, beds, bed_count, comment, changed_by, changed_at FROM responses JOIN invitations ON invitations.id = responses.invitation_id WHERE invitations.gathering_id = $1",
    )
    .bind(gathering_id)
    .fetch_all(connection)
    .await?
    .into_iter()
    .map(|response| (response.invitation_id, response))
    .collect();
    let mut totals = Totals::default();
    let rows = invitations
        .into_iter()
        .map(|invitation| {
            let response = responses.get(&invitation.id);
            let row = OverviewRow {
                invitation_id: invitation.id,
                first_name: invitation.first_name,
                last_name: invitation.last_name,
                status: response.map_or(InvitationStatus::Pending, |response| {
                    match response.status {
                        ResponseStatus::Accepted => InvitationStatus::Accepted,
                        ResponseStatus::Tentative => InvitationStatus::Tentative,
                        ResponseStatus::Declined => InvitationStatus::Declined,
                    }
                }),
                arrival: response.and_then(|response| response.arrival),
                departure: response.and_then(|response| response.departure),
                beds: response.and_then(|response| response.beds),
                bed_count: response.and_then(|response| response.bed_count),
                comment: response.and_then(|response| response.comment.clone()),
                changed_by: response.map(|response| response.changed_by),
                changed_at: response
                    .map(|response| state.time_zone.wall_clock(response.changed_at)),
                outside_span: response.is_some_and(|response| {
                    [response.arrival, response.departure]
                        .into_iter()
                        .flatten()
                        .any(|time| !within(time, &gathering))
                }),
                link_active: invitation.link_active,
                mail: invitation.mail,
            };
            count(&mut totals, &row);
            row
        })
        .collect();
    Ok(Some(Overview { rows, totals }))
}

fn count(totals: &mut Totals, row: &OverviewRow) {
    match row.status {
        InvitationStatus::Pending => totals.pending += 1,
        InvitationStatus::Accepted => totals.accepted += 1,
        InvitationStatus::Tentative => totals.tentative += 1,
        InvitationStatus::Declined => totals.declined += 1,
    }
    let count = i64::from(row.bed_count.unwrap_or(0));
    match row.beds {
        Some(Beds::Need) => totals.beds_needed += count,
        Some(Beds::Offer) => totals.beds_offered += count,
        _ => {}
    }
}

#[utoipa::path(
    put,
    path = "/api/invitations/{id}/response",
    params(("id" = i64, Path, description = "Id of the invitation")),
    request_body = ResponseValues,
    responses(
        (status = 200, description = "Response stored on the member's behalf", body = ResponseValues),
        (status = 401, description = "No valid organizer session"),
        (status = 404, description = "No such invitation"),
        (status = 422, description = "Response invalid", body = crate::validation::ValidationErrors),
    )
)]
async fn answer_for_member(
    _: Organizer,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(values): Json<ResponseValues>,
) -> Response {
    let Ok(mut connection) = state.pool.acquire().await else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    match invitations::gathering_of(&mut connection, id, state.time_zone.today()).await {
        Ok(Some(gathering)) => save(&state, id, &gathering, &values, ChangedBy::Organizer).await,
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

fn link_state(gathering: &Gathering, today: Date) -> LinkState {
    if gathering.cancelled {
        LinkState::Cancelled
    } else if gathering.ended {
        LinkState::Ended
    } else if gathering.has_begun(today) {
        LinkState::Begun
    } else {
        LinkState::Open
    }
}

/// Expects the secret of the personal link as `Authorization: Bearer <secret>`.
#[utoipa::path(
    put,
    path = "/api/invitation/response",
    request_body = ResponseValues,
    responses(
        (status = 200, description = "Response stored, as the page shows it now", body = ResponseValues),
        (status = 404, description = "The link is unknown or revoked"),
        (status = 422, description = "Response invalid, or the gathering has begun, ended or was cancelled", body = crate::validation::ValidationErrors),
    )
)]
async fn answer(
    State(state): State<AppState>,
    link: PersonalLink,
    Json(values): Json<ResponseValues>,
) -> Response {
    let code = match link_state(&link.gathering, state.time_zone.today()) {
        LinkState::Open => None,
        LinkState::Begun => Some(ErrorCode::Begun),
        LinkState::Ended => Some(ErrorCode::Ended),
        LinkState::Cancelled => Some(ErrorCode::Cancelled),
    };
    if let Some(code) = code {
        return invalid(vec![field_error("gathering", code)]);
    }
    save(
        &state,
        link.invitation_id,
        &link.gathering,
        &values,
        ChangedBy::Member,
    )
    .await
}

async fn save(
    state: &AppState,
    invitation_id: i64,
    gathering: &Gathering,
    values: &ResponseValues,
    changed_by: ChangedBy,
) -> Response {
    let values = values.normalized();
    let errors = values.validate(gathering);
    if !errors.is_empty() {
        return invalid(errors);
    }
    let Ok(mut connection) = state.pool.acquire().await else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    match store(&mut connection, invitation_id, &values, changed_by).await {
        Ok(()) => Json(values).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// Expects the secret of the personal link as `Authorization: Bearer <secret>`.
#[utoipa::path(
    get,
    path = "/api/invitation",
    responses(
        (status = 200, description = "The gathering and the state of the link", body = InvitationPage),
        (status = 404, description = "The link is unknown or revoked"),
    )
)]
async fn invitation_page(State(state): State<AppState>, link: PersonalLink) -> Response {
    let Ok(mut connection) = state.pool.acquire().await else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    let Ok(response) = stored(&mut connection, link.invitation_id).await else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    let gathering = link.gathering;
    let link_state = link_state(&gathering, state.time_zone.today());
    let attendees = if matches!(link_state, LinkState::Open | LinkState::Begun) {
        match attendees(&mut connection, gathering.id).await {
            Ok(attendees) => Some(attendees),
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    } else {
        None
    };
    let calendar = offers_calendar(&gathering, response.as_ref());
    Json(InvitationPage {
        first_name: link.member.first_name,
        state: link_state,
        gathering: InvitedGathering {
            title: gathering.title,
            location: gathering.location,
            description: gathering.description,
            first_day: gathering.first_day,
            last_day: gathering.last_day,
        },
        calendar,
        response,
        attendees,
    })
    .into_response()
}

/// Whether a member who comes or may come can still add the gathering to a calendar, also when it is cancelled.
fn offers_calendar(gathering: &Gathering, response: Option<&ResponseValues>) -> bool {
    !gathering.ended
        && response.is_some_and(|response| {
            matches!(
                response.status,
                ResponseStatus::Accepted | ResponseStatus::Tentative
            )
        })
}

/// Expects the secret of the personal link as `Authorization: Bearer <secret>`.
#[utoipa::path(
    get,
    path = "/api/invitation/calendar",
    responses(
        (status = 200, description = "The calendar file of the gathering", body = String, content_type = "text/calendar"),
        (status = 404, description = "The link is unknown or revoked, or the page offers no calendar file"),
    )
)]
async fn calendar_file(State(state): State<AppState>, link: PersonalLink) -> Response {
    let Ok(mut connection) = state.pool.acquire().await else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    let Ok(response) = stored(&mut connection, link.invitation_id).await else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    if !offers_calendar(&link.gathering, response.as_ref()) {
        return StatusCode::NOT_FOUND.into_response();
    }
    let Ok(revision) = gatherings::revision(&mut connection, link.gathering.id).await else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    (
        [
            (header::CONTENT_TYPE, "text/calendar; charset=utf-8"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"gathering.ics\"",
            ),
        ],
        calendar::file(&link.gathering, &revision),
    )
        .into_response()
}

#[cfg(test)]
mod integration {
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode, header};
    use serde_json::{Value, json};
    use sqlx::PgPool;
    use tower::ServiceExt;

    use time::Date;

    use super::expire;
    use crate::access::testing::{session_cookie, signed_in_organizer};
    use crate::api;
    use crate::configuration::{Retention, RetentionPeriod};
    use crate::database::testing::fresh_database;
    use crate::invitations::testing::{gathering, member, send};

    /// Invites one member to a gathering through the API and returns the secret from the queued mail.
    async fn invited(pool: &PgPool, first: i32, last: i32) -> String {
        let (_, cookie) = signed_in_organizer(pool).await;
        let clara = member(pool, "Clara", "de").await;
        let weekend = gathering(pool, "Probenwochenende", first, last).await;
        sqlx::query("UPDATE gatherings SET first_day = $2, last_day = $3 WHERE id = $1")
            .bind(weekend)
            .bind(api::testing::day(10))
            .bind(api::testing::day(12))
            .execute(pool)
            .await
            .unwrap();
        let (status, _) = send(
            api::testing::state(pool.clone()),
            &cookie,
            "POST",
            &format!("/api/gatherings/{weekend}/invitations"),
            &json!({ "memberIds": [clara] }),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        sqlx::query("UPDATE gatherings SET first_day = $2, last_day = $3 WHERE id = $1")
            .bind(weekend)
            .bind(api::testing::day(first))
            .bind(api::testing::day(last))
            .execute(pool)
            .await
            .unwrap();
        let text: String =
            sqlx::query_scalar("SELECT text_body FROM mail_outbox WHERE invitation_id IS NOT NULL")
                .fetch_one(pool)
                .await
                .unwrap();
        let start = text.find("/invitation#").unwrap() + "/invitation#".len();
        text[start..start + 64].to_owned()
    }

    async fn open(pool: &PgPool, secret: Option<&str>) -> (StatusCode, Value) {
        let mut request = Request::get("/api/invitation");
        if let Some(secret) = secret {
            request = request.header(header::AUTHORIZATION, format!("Bearer {secret}"));
        }
        let response = api::router(api::testing::state(pool.clone()))
            .oneshot(request.body(Body::empty()).unwrap())
            .await
            .unwrap();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        (
            status,
            serde_json::from_slice(&bytes).unwrap_or(Value::Null),
        )
    }

    async fn answer(pool: &PgPool, secret: &str, body: &Value) -> (StatusCode, Value) {
        let request = Request::put("/api/invitation/response")
            .header(header::AUTHORIZATION, format!("Bearer {secret}"))
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

    async fn first_day(pool: &PgPool) -> String {
        let day: time::Date = sqlx::query_scalar("SELECT first_day FROM gatherings")
            .fetch_one(pool)
            .await
            .unwrap();
        day.to_string()
    }

    async fn row(pool: &PgPool) -> Value {
        let text: String = sqlx::query_scalar(
            "SELECT row_to_json(stored)::text FROM (SELECT status, to_char(arrival, 'YYYY-MM-DD\"T\"HH24:MI') AS arrival, to_char(departure, 'YYYY-MM-DD\"T\"HH24:MI') AS departure, beds, bed_count AS \"bedCount\", comment, changed_by AS \"changedBy\" FROM responses) stored",
        )
        .fetch_one(pool)
        .await
        .unwrap();
        serde_json::from_str(&text).unwrap()
    }

    async fn rejected(pool: &PgPool, secret: &str, body: Value) -> Value {
        let (status, errors) = answer(pool, secret, &body).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
        errors
    }

    #[tokio::test]
    async fn accept() {
        let database = fresh_database().await;
        let pool = database.pool();
        let secret = invited(&pool, 10, 12).await;
        let day = first_day(&pool).await;
        let body =
            json!({ "status": "accepted", "arrival": format!("{day}T18:00"), "beds": "none" });
        let (status, stored) = answer(&pool, &secret, &body).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(stored["arrival"], format!("{day}T18:00"));
        assert_eq!(row(&pool).await["status"], "accepted");
        assert_eq!(row(&pool).await["changedBy"], "member");
    }

    #[tokio::test]
    async fn decline() {
        let database = fresh_database().await;
        let pool = database.pool();
        let secret = invited(&pool, 10, 12).await;
        let day = first_day(&pool).await;
        let body = json!({ "status": "declined", "arrival": format!("{day}T18:00"), "beds": "need", "bedCount": 2 });
        assert_eq!(answer(&pool, &secret, &body).await.0, StatusCode::OK);
        let stored = row(&pool).await;
        assert_eq!(stored["status"], "declined");
        assert_eq!(
            (
                stored["arrival"].clone(),
                stored["beds"].clone(),
                stored["bedCount"].clone()
            ),
            (Value::Null, Value::Null, Value::Null)
        );
    }

    #[tokio::test]
    async fn period_outside_the_gathering() {
        let database = fresh_database().await;
        let pool = database.pool();
        let secret = invited(&pool, 10, 12).await;
        let before: time::Date = sqlx::query_scalar("SELECT first_day - 1 FROM gatherings")
            .fetch_one(&pool)
            .await
            .unwrap();
        let errors = rejected(
            &pool,
            &secret,
            json!({ "status": "accepted", "arrival": format!("{before}T18:00"), "beds": "none" }),
        )
        .await;
        assert_eq!(
            errors,
            json!({ "errors": [{ "field": "arrival", "code": "invalid" }] })
        );
    }

    #[tokio::test]
    async fn leave_before_arrival() {
        let database = fresh_database().await;
        let pool = database.pool();
        let secret = invited(&pool, 10, 12).await;
        let day = first_day(&pool).await;
        let errors = rejected(&pool, &secret, json!({ "status": "accepted", "arrival": format!("{day}T18:00"), "departure": format!("{day}T17:30"), "beds": "none" })).await;
        assert_eq!(
            errors,
            json!({ "errors": [{ "field": "departure", "code": "beforeArrival" }] })
        );
    }

    #[tokio::test]
    async fn time_between_the_steps() {
        let database = fresh_database().await;
        let pool = database.pool();
        let secret = invited(&pool, 10, 12).await;
        let day = first_day(&pool).await;
        let errors = rejected(
            &pool,
            &secret,
            json!({ "status": "accepted", "arrival": format!("{day}T18:10"), "beds": "none" }),
        )
        .await;
        assert_eq!(
            errors,
            json!({ "errors": [{ "field": "arrival", "code": "invalid" }] })
        );
    }

    #[tokio::test]
    async fn offer_beds() {
        let database = fresh_database().await;
        let pool = database.pool();
        let secret = invited(&pool, 10, 12).await;
        assert_eq!(
            answer(
                &pool,
                &secret,
                &json!({ "status": "tentative", "beds": "offer", "bedCount": 2 })
            )
            .await
            .0,
            StatusCode::OK
        );
        let stored = row(&pool).await;
        assert_eq!(
            (stored["beds"].clone(), stored["bedCount"].clone()),
            (json!("offer"), json!(2))
        );
    }

    #[tokio::test]
    async fn need_without_a_count() {
        let database = fresh_database().await;
        let pool = database.pool();
        let secret = invited(&pool, 10, 12).await;
        let errors = rejected(
            &pool,
            &secret,
            json!({ "status": "accepted", "beds": "need" }),
        )
        .await;
        assert_eq!(
            errors,
            json!({ "errors": [{ "field": "bedCount", "code": "required" }] })
        );
    }

    #[tokio::test]
    async fn more_than_20_beds() {
        let database = fresh_database().await;
        let pool = database.pool();
        let secret = invited(&pool, 10, 12).await;
        let errors = rejected(
            &pool,
            &secret,
            json!({ "status": "accepted", "beds": "offer", "bedCount": 21 }),
        )
        .await;
        assert_eq!(
            errors,
            json!({ "errors": [{ "field": "bedCount", "code": "tooLarge" }] })
        );
    }

    #[tokio::test]
    async fn comment_with_a_decline() {
        let database = fresh_database().await;
        let pool = database.pool();
        let secret = invited(&pool, 10, 12).await;
        assert_eq!(
            answer(
                &pool,
                &secret,
                &json!({ "status": "declined", "comment": " Urlaub " })
            )
            .await
            .0,
            StatusCode::OK
        );
        assert_eq!(row(&pool).await["comment"], "Urlaub");
        let errors = rejected(
            &pool,
            &secret,
            json!({ "status": "declined", "comment": "x".repeat(2001) }),
        )
        .await;
        assert_eq!(
            errors,
            json!({ "errors": [{ "field": "comment", "code": "tooLong" }] })
        );
    }

    #[tokio::test]
    async fn coming_without_beds() {
        let database = fresh_database().await;
        let pool = database.pool();
        let secret = invited(&pool, 10, 12).await;
        let errors = rejected(&pool, &secret, json!({ "status": "accepted" })).await;
        assert_eq!(
            errors,
            json!({ "errors": [{ "field": "beds", "code": "required" }] })
        );
    }

    async fn invitation_id(pool: &PgPool) -> i64 {
        sqlx::query_scalar("SELECT id FROM invitations")
            .fetch_one(pool)
            .await
            .unwrap()
    }

    async fn answer_as_organizer(pool: &PgPool, body: &Value) -> (StatusCode, Value) {
        let organizer: i64 = sqlx::query_scalar("SELECT id FROM members WHERE organizer")
            .fetch_one(pool)
            .await
            .unwrap();
        let cookie = crate::access::testing::session_cookie(pool, organizer).await;
        let id = invitation_id(pool).await;
        send(
            api::testing::state(pool.clone()),
            &cookie,
            "PUT",
            &format!("/api/invitations/{id}/response"),
            body,
        )
        .await
    }

    async fn move_start(pool: &PgPool, first: i32, last: i32) {
        sqlx::query("UPDATE gatherings SET first_day = $1, last_day = $2")
            .bind(api::testing::day(first))
            .bind(api::testing::day(last))
            .execute(pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn change_from_tentative_to_accepted() {
        let database = fresh_database().await;
        let pool = database.pool();
        let secret = invited(&pool, 10, 12).await;
        answer(&pool, &secret, &json!({ "status": "tentative", "beds": "need", "bedCount": 1, "comment": "Vielleicht später" })).await;
        let (_, page) = open(&pool, Some(&secret)).await;
        assert_eq!(page["response"]["status"], "tentative");
        assert_eq!(page["response"]["bedCount"], 1);
        assert_eq!(page["response"]["comment"], "Vielleicht später");
        let (status, _) = answer(
            &pool,
            &secret,
            &json!({ "status": "accepted", "beds": "need", "bedCount": 1 }),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(row(&pool).await["status"], "accepted");
    }

    #[tokio::test]
    async fn open_the_link_after_the_start() {
        let database = fresh_database().await;
        let pool = database.pool();
        let secret = invited(&pool, 10, 12).await;
        answer(
            &pool,
            &secret,
            &json!({ "status": "accepted", "beds": "none" }),
        )
        .await;
        move_start(&pool, 0, 2).await;
        let (_, page) = open(&pool, Some(&secret)).await;
        assert_eq!(page["state"], "begun");
        assert_eq!(page["response"]["status"], "accepted");
        let errors = rejected(&pool, &secret, json!({ "status": "declined" })).await;
        assert_eq!(
            errors,
            json!({ "errors": [{ "field": "gathering", "code": "begun" }] })
        );
    }

    #[tokio::test]
    async fn answer_on_a_members_behalf() {
        let database = fresh_database().await;
        let pool = database.pool();
        invited(&pool, 10, 12).await;
        move_start(&pool, -1, 1).await;
        let (status, _) =
            answer_as_organizer(&pool, &json!({ "status": "accepted", "beds": "none" })).await;
        assert_eq!(status, StatusCode::OK);
        let stored = row(&pool).await;
        assert_eq!(
            (stored["status"].clone(), stored["changedBy"].clone()),
            (json!("accepted"), json!("organizer"))
        );
    }

    #[tokio::test]
    async fn member_opens_the_link_afterwards() {
        let database = fresh_database().await;
        let pool = database.pool();
        let secret = invited(&pool, 10, 12).await;
        answer_as_organizer(
            &pool,
            &json!({ "status": "tentative", "beds": "offer", "bedCount": 3 }),
        )
        .await;
        let (_, page) = open(&pool, Some(&secret)).await;
        assert_eq!(page["state"], "open");
        assert_eq!(page["response"]["status"], "tentative");
        assert_eq!(page["response"]["bedCount"], 3);
    }

    /// Invites members with the given responses next to Clara, who opens her link. `None` leaves an invitation pending.
    async fn gathering_with(
        pool: &PgPool,
        responses: &[(&str, Option<&str>, Option<&str>)],
    ) -> String {
        let secret = invited(pool, 10, 12).await;
        let gathering_id: i64 = sqlx::query_scalar("SELECT id FROM gatherings")
            .fetch_one(pool)
            .await
            .unwrap();
        for (name, status, comment) in responses {
            let member_id = member(pool, name, "de").await;
            let invitation_id: i64 = sqlx::query_scalar(
                "INSERT INTO invitations (gathering_id, member_id) VALUES ($1, $2) RETURNING id",
            )
            .bind(gathering_id)
            .bind(member_id)
            .fetch_one(pool)
            .await
            .unwrap();
            if let Some(status) = status {
                let beds = if *status == "declined" {
                    None
                } else {
                    Some("none")
                };
                sqlx::query("INSERT INTO responses (invitation_id, status, beds, comment, changed_by) VALUES ($1, $2, $3, $4, 'member')")
                    .bind(invitation_id)
                    .bind(status)
                    .bind(beds)
                    .bind(comment)
                    .execute(pool)
                    .await
                    .unwrap();
            }
        }
        secret
    }

    #[tokio::test]
    async fn member_looks_at_attendees() {
        let database = fresh_database().await;
        let pool = database.pool();
        let secret = gathering_with(
            &pool,
            &[
                ("Anna", Some("accepted"), None),
                ("Ben", Some("accepted"), None),
                ("Clemens", Some("accepted"), None),
                ("Felix", Some("tentative"), None),
                ("Moritz", Some("declined"), None),
                ("Sophie", Some("declined"), None),
            ],
        )
        .await;
        let (_, page) = open(&pool, Some(&secret)).await;
        assert_eq!(
            page["attendees"],
            json!({
                "coming": [
                    { "name": "Anna Tester", "status": "accepted" },
                    { "name": "Ben Tester", "status": "accepted" },
                    { "name": "Clemens Tester", "status": "accepted" },
                    { "name": "Felix Tester", "status": "tentative" },
                ],
                "declined": 2,
                "pending": 1,
            })
        );
    }

    #[tokio::test]
    async fn comment_of_another_member() {
        let database = fresh_database().await;
        let pool = database.pool();
        let secret = gathering_with(
            &pool,
            &[("Anna", Some("accepted"), Some("Ich bringe Kuchen mit"))],
        )
        .await;
        let (_, page) = open(&pool, Some(&secret)).await;
        let text = page.to_string();
        assert!(!text.contains("Kuchen"), "{text}");
        assert!(!text.contains("example.org"), "{text}");
    }

    #[tokio::test]
    async fn no_attendees_after_the_gathering() {
        let database = fresh_database().await;
        let pool = database.pool();
        let secret = gathering_with(&pool, &[("Anna", Some("accepted"), None)]).await;
        move_start(&pool, -3, -1).await;
        let (_, page) = open(&pool, Some(&secret)).await;
        assert_eq!(page["attendees"], Value::Null);
    }

    async fn overview_of(pool: &PgPool) -> Value {
        let organizer: i64 = sqlx::query_scalar("SELECT id FROM members WHERE organizer")
            .fetch_one(pool)
            .await
            .unwrap();
        let cookie = crate::access::testing::session_cookie(pool, organizer).await;
        let gathering_id: i64 = sqlx::query_scalar("SELECT id FROM gatherings")
            .fetch_one(pool)
            .await
            .unwrap();
        let (status, overview) = send(
            api::testing::state(pool.clone()),
            &cookie,
            "GET",
            &format!("/api/gatherings/{gathering_id}/overview"),
            &Value::Null,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        overview
    }

    #[tokio::test]
    async fn plan_beds() {
        let database = fresh_database().await;
        let pool = database.pool();
        gathering_with(
            &pool,
            &[
                ("Anna", None, None),
                ("Ben", None, None),
                ("Felix", None, None),
                ("Moritz", Some("declined"), None),
            ],
        )
        .await;
        for (name, beds, count) in [
            ("Anna", "need", 1),
            ("Ben", "need", 1),
            ("Felix", "offer", 3),
        ] {
            sqlx::query("INSERT INTO responses (invitation_id, status, beds, bed_count, changed_by) SELECT invitations.id, 'accepted', $2, $3, 'member' FROM invitations JOIN members ON members.id = invitations.member_id WHERE members.first_name = $1")
                .bind(name)
                .bind(beds)
                .bind(count)
                .execute(&pool)
                .await
                .unwrap();
        }
        let overview = overview_of(&pool).await;
        assert_eq!(
            overview["totals"],
            json!({ "accepted": 3, "tentative": 0, "declined": 1, "pending": 1, "bedsNeeded": 2, "bedsOffered": 3 })
        );
        let anna = overview["rows"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["firstName"] == "Anna")
            .unwrap()
            .clone();
        assert_eq!(
            (
                anna["status"].clone(),
                anna["beds"].clone(),
                anna["bedCount"].clone(),
                anna["changedBy"].clone()
            ),
            (json!("accepted"), json!("need"), json!(1), json!("member"))
        );
        assert!(anna["changedAt"].as_str().unwrap().len() == 16, "{anna}");
    }

    #[tokio::test]
    async fn no_period_stated() {
        let database = fresh_database().await;
        let pool = database.pool();
        let secret = invited(&pool, 10, 12).await;
        answer(
            &pool,
            &secret,
            &json!({ "status": "accepted", "beds": "none" }),
        )
        .await;
        let overview = overview_of(&pool).await;
        let clara = &overview["rows"][0];
        assert_eq!(
            (
                clara["arrival"].clone(),
                clara["departure"].clone(),
                clara["outsideSpan"].clone()
            ),
            (Value::Null, Value::Null, json!(false))
        );
    }

    #[tokio::test]
    async fn shorten_a_gathering() {
        let database = fresh_database().await;
        let pool = database.pool();
        let secret = invited(&pool, 10, 12).await;
        let day = first_day(&pool).await;
        answer(
            &pool,
            &secret,
            &json!({ "status": "accepted", "arrival": format!("{day}T18:00"), "beds": "none" }),
        )
        .await;
        let organizer: i64 = sqlx::query_scalar("SELECT id FROM members WHERE organizer")
            .fetch_one(&pool)
            .await
            .unwrap();
        let cookie = crate::access::testing::session_cookie(&pool, organizer).await;
        let gathering_id: i64 = sqlx::query_scalar("SELECT id FROM gatherings")
            .fetch_one(&pool)
            .await
            .unwrap();
        let shorter = json!({ "title": "Probenwochenende", "location": "Kloster Drübeck", "firstDay": api::testing::day(11).to_string(), "lastDay": api::testing::day(12).to_string() });
        let (status, _) = send(
            api::testing::state(pool.clone()),
            &cookie,
            "PUT",
            &format!("/api/gatherings/{gathering_id}"),
            &shorter,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        let overview = overview_of(&pool).await;
        assert_eq!(overview["rows"][0]["status"], "accepted");
        assert_eq!(overview["rows"][0]["arrival"], format!("{day}T18:00"));
        assert_eq!(overview["rows"][0]["outsideSpan"], true);
    }

    #[tokio::test]
    async fn failed_mail_appears_at_its_invitation() {
        let database = fresh_database().await;
        let pool = database.pool();
        invited(&pool, 10, 12).await;
        sqlx::query("UPDATE mail_outbox SET failed_at = now(), attempts = 8, recipient = NULL, subject = NULL, text_body = NULL, html_body = NULL")
            .execute(&pool)
            .await
            .unwrap();
        let overview = overview_of(&pool).await;
        assert_eq!(overview["rows"][0]["mail"], "failed");
        assert_eq!(overview["rows"][0]["linkActive"], true);
    }

    /// A gathering that ended the given number of days ago with one accepted response that carries a comment.
    async fn ended_with_comment(pool: &PgPool, days_ago: i32) {
        let gathering = gathering(pool, "Herbstcamp", -days_ago - 2, -days_ago).await;
        let anna = member(pool, "Anna", "de").await;
        let invitation: i64 = sqlx::query_scalar(
            "INSERT INTO invitations (gathering_id, member_id) VALUES ($1, $2) RETURNING id",
        )
        .bind(gathering)
        .bind(anna)
        .fetch_one(pool)
        .await
        .unwrap();
        sqlx::query("INSERT INTO responses (invitation_id, status, beds, comment, changed_by) VALUES ($1, 'accepted', 'none', 'Kuchen', 'member')")
            .bind(invitation)
            .execute(pool)
            .await
            .unwrap();
    }

    async fn left(pool: &PgPool) -> (i64, i64, i64, i64) {
        sqlx::query_as("SELECT (SELECT count(*) FROM gatherings), (SELECT count(*) FROM invitations), (SELECT count(*) FROM responses), (SELECT count(*) FROM responses WHERE comment IS NOT NULL)")
            .fetch_one(pool)
            .await
            .unwrap()
    }

    fn retention(comments: RetentionPeriod, responses: RetentionPeriod) -> Retention {
        Retention {
            comments,
            responses,
        }
    }

    fn today() -> Date {
        api::testing::day(0)
    }

    #[tokio::test]
    async fn default_retention() {
        let database = fresh_database().await;
        let pool = database.pool();
        ended_with_comment(&pool, 31).await;
        expire(
            &pool,
            retention(RetentionPeriod::Days(30), RetentionPeriod::Days(365)),
            today(),
        )
        .await
        .unwrap();
        assert_eq!(left(&pool).await, (1, 1, 1, 0));
    }

    #[tokio::test]
    async fn keep_everything() {
        let database = fresh_database().await;
        let pool = database.pool();
        ended_with_comment(&pool, 730).await;
        expire(
            &pool,
            retention(RetentionPeriod::Never, RetentionPeriod::Never),
            today(),
        )
        .await
        .unwrap();
        assert_eq!(left(&pool).await, (1, 1, 1, 1));
    }

    #[tokio::test]
    async fn responses_expire() {
        let database = fresh_database().await;
        let pool = database.pool();
        ended_with_comment(&pool, 91).await;
        expire(
            &pool,
            retention(RetentionPeriod::Never, RetentionPeriod::Days(90)),
            today(),
        )
        .await
        .unwrap();
        assert_eq!(left(&pool).await, (1, 0, 0, 0));
    }

    #[tokio::test]
    async fn comments_expire() {
        let database = fresh_database().await;
        let pool = database.pool();
        ended_with_comment(&pool, 31).await;
        expire(
            &pool,
            retention(RetentionPeriod::Days(30), RetentionPeriod::Never),
            today(),
        )
        .await
        .unwrap();
        assert_eq!(left(&pool).await, (1, 1, 1, 0));
    }

    #[tokio::test]
    async fn comments_stay_within_their_period() {
        let database = fresh_database().await;
        let pool = database.pool();
        ended_with_comment(&pool, 30).await;
        expire(
            &pool,
            retention(RetentionPeriod::Days(30), RetentionPeriod::Days(365)),
            today(),
        )
        .await
        .unwrap();
        assert_eq!(left(&pool).await, (1, 1, 1, 1));
    }

    #[tokio::test]
    async fn ended_sessions_and_used_links_are_gone() {
        let database = fresh_database().await;
        let pool = database.pool();
        let (organizer, _) = crate::access::testing::signed_in_organizer(&pool).await;
        crate::access::testing::session_cookie(&pool, organizer).await;
        sqlx::query("UPDATE sessions SET expires_at = now() - interval '1 second' WHERE id = (SELECT min(id) FROM sessions)").execute(&pool).await.unwrap();
        for (expires, used) in [
            ("now() + interval '1 hour'", "NULL"),
            ("now() + interval '1 hour'", "now()"),
            ("now() - interval '1 second'", "NULL"),
        ] {
            sqlx::query(sqlx::AssertSqlSafe(format!("INSERT INTO sign_in_links (member_id, secret_hash, expires_at, used_at) VALUES ($1, gen_random_uuid()::text::bytea, {expires}, {used})")))
                .bind(organizer)
                .execute(&pool)
                .await
                .unwrap();
        }
        expire(
            &pool,
            retention(RetentionPeriod::Never, RetentionPeriod::Never),
            today(),
        )
        .await
        .unwrap();
        let (sessions, links): (i64, i64) = sqlx::query_as(
            "SELECT (SELECT count(*) FROM sessions), (SELECT count(*) FROM sign_in_links)",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!((sessions, links), (1, 1));
    }

    #[tokio::test]
    async fn open_a_valid_link() {
        let database = fresh_database().await;
        let pool = database.pool();
        let secret = invited(&pool, 10, 12).await;
        let (status, page) = open(&pool, Some(&secret)).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(page["firstName"], "Clara");
        assert_eq!(page["gathering"]["title"], "Probenwochenende");
        assert_eq!(page["state"], "open");
    }

    #[tokio::test]
    async fn guessed_link() {
        let database = fresh_database().await;
        let pool = database.pool();
        invited(&pool, 10, 12).await;
        for secret in [Some("0".repeat(64)), Some("not-even-hex".to_owned()), None] {
            let (status, body) = open(&pool, secret.as_deref()).await;
            assert_eq!(status, StatusCode::NOT_FOUND);
            assert_eq!(body, Value::Null, "the answer must reveal nothing");
        }
    }

    #[tokio::test]
    async fn open_after_the_gathering() {
        let database = fresh_database().await;
        let pool = database.pool();
        let secret = invited(&pool, -3, -1).await;
        let (status, page) = open(&pool, Some(&secret)).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(page["state"], "ended");
    }

    #[tokio::test]
    async fn member_opens_a_cancelled_gathering() {
        let database = fresh_database().await;
        let pool = database.pool();
        let secret = invited(&pool, 10, 12).await;
        sqlx::query("UPDATE gatherings SET cancelled_at = now()")
            .execute(&pool)
            .await
            .unwrap();
        let (status, page) = open(&pool, Some(&secret)).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(page["state"], "cancelled");
    }

    #[tokio::test]
    async fn open_on_the_first_day() {
        let database = fresh_database().await;
        let pool = database.pool();
        let secret = invited(&pool, 0, 2).await;
        let (_, page) = open(&pool, Some(&secret)).await;
        assert_eq!(page["state"], "begun");
    }

    async fn download(pool: &PgPool, secret: Option<&str>) -> (StatusCode, String, String) {
        let mut request = Request::get("/api/invitation/calendar");
        if let Some(secret) = secret {
            request = request.header(header::AUTHORIZATION, format!("Bearer {secret}"));
        }
        let response = api::router(api::testing::state(pool.clone()))
            .oneshot(request.body(Body::empty()).unwrap())
            .await
            .unwrap();
        let status = response.status();
        let header_text = |name| {
            response
                .headers()
                .get(name)
                .map(|value: &header::HeaderValue| value.to_str().unwrap().to_owned())
                .unwrap_or_default()
        };
        let content_type = header_text(header::CONTENT_TYPE);
        let disposition = header_text(header::CONTENT_DISPOSITION);
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body = String::from_utf8(bytes.to_vec()).unwrap();
        assert!(
            status != StatusCode::OK || disposition == "attachment; filename=\"gathering.ics\""
        );
        (status, content_type, body)
    }

    async fn respond(pool: &PgPool, secret: &str, status: &str) {
        let body = if status == "declined" {
            json!({ "status": status })
        } else {
            json!({ "status": status, "beds": "none" })
        };
        assert_eq!(answer(pool, secret, &body).await.0, StatusCode::OK);
    }

    /// A session of the organizer that `invited` stored and the id of its gathering.
    async fn organizer_and_gathering(pool: &PgPool) -> (String, i64) {
        let (organizer, gathering): (i64, i64) = sqlx::query_as(
            "SELECT (SELECT id FROM members WHERE organizer), (SELECT id FROM gatherings)",
        )
        .fetch_one(pool)
        .await
        .unwrap();
        (session_cookie(pool, organizer).await, gathering)
    }

    #[tokio::test]
    async fn accepted_member_adds_the_gathering() {
        let database = fresh_database().await;
        let pool = database.pool();
        let secret = invited(&pool, 10, 12).await;
        respond(&pool, &secret, "accepted").await;
        assert_eq!(open(&pool, Some(&secret)).await.1["calendar"], true);
        let (status, content_type, file) = download(&pool, Some(&secret)).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(content_type, "text/calendar; charset=utf-8");
        assert!(file.contains("SUMMARY:Probenwochenende\r\n"), "{file}");
        assert!(file.contains("STATUS:CONFIRMED\r\n"), "{file}");
        assert!(file.contains("SEQUENCE:0\r\n"), "{file}");
        assert!(
            !file.contains("Clara"),
            "the file holds nothing of the member"
        );
    }

    #[tokio::test]
    async fn tentative_member_of_a_cancelled_gathering() {
        let database = fresh_database().await;
        let pool = database.pool();
        let secret = invited(&pool, 10, 12).await;
        respond(&pool, &secret, "tentative").await;
        let (cookie, id) = organizer_and_gathering(&pool).await;
        let (status, _) = send(
            api::testing::state(pool.clone()),
            &cookie,
            "POST",
            &format!("/api/gatherings/{id}/cancellation"),
            &Value::Null,
        )
        .await;
        assert_eq!(status, StatusCode::NO_CONTENT);
        assert_eq!(open(&pool, Some(&secret)).await.1["calendar"], true);
        let (status, _, file) = download(&pool, Some(&secret)).await;
        assert_eq!(status, StatusCode::OK);
        assert!(file.contains("STATUS:CANCELLED\r\n"), "{file}");
        assert!(file.contains("SEQUENCE:1\r\n"), "{file}");
    }

    #[tokio::test]
    async fn entry_after_two_changes() {
        let database = fresh_database().await;
        let pool = database.pool();
        let secret = invited(&pool, 10, 12).await;
        respond(&pool, &secret, "accepted").await;
        let uid_before = download(&pool, Some(&secret))
            .await
            .2
            .lines()
            .find(|line| line.starts_with("UID:"))
            .unwrap()
            .to_owned();
        let (cookie, id) = organizer_and_gathering(&pool).await;
        let change = json!({ "title": "Probenwochenende", "location": "Kloster Drübeck", "firstDay": api::testing::day(10).to_string(), "lastDay": api::testing::day(13).to_string() });
        for _ in 0..2 {
            let (status, _) = send(
                api::testing::state(pool.clone()),
                &cookie,
                "PUT",
                &format!("/api/gatherings/{id}"),
                &change,
            )
            .await;
            assert_eq!(status, StatusCode::OK);
        }
        let (changed, created): (String, String) = sqlx::query_as(
            "SELECT to_char(changed_at AT TIME ZONE 'UTC', 'YYYYMMDD\"T\"HH24MISS\"Z\"'), to_char(created_at AT TIME ZONE 'UTC', 'YYYYMMDD\"T\"HH24MISS\"Z\"') FROM gatherings",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        let file = download(&pool, Some(&secret)).await.2;
        assert!(file.contains("SEQUENCE:2\r\n"), "{file}");
        assert!(file.contains(&format!("DTSTAMP:{changed}\r\n")), "{file}");
        assert!(
            file.contains(&format!("LAST-MODIFIED:{changed}\r\n")),
            "{file}"
        );
        assert!(file.contains(&format!("CREATED:{created}\r\n")), "{file}");
        assert!(file.contains(&format!("{uid_before}\r\n")), "{file}");
    }

    #[tokio::test]
    async fn declined_or_pending_member() {
        let database = fresh_database().await;
        let pool = database.pool();
        let secret = invited(&pool, 10, 12).await;
        assert_eq!(open(&pool, Some(&secret)).await.1["calendar"], false);
        assert_eq!(
            download(&pool, Some(&secret)).await.0,
            StatusCode::NOT_FOUND
        );
        respond(&pool, &secret, "declined").await;
        assert_eq!(open(&pool, Some(&secret)).await.1["calendar"], false);
        let (status, _, body) = download(&pool, Some(&secret)).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body, "");
    }

    #[tokio::test]
    async fn calendar_after_the_gathering() {
        let database = fresh_database().await;
        let pool = database.pool();
        let secret = invited(&pool, 10, 12).await;
        respond(&pool, &secret, "accepted").await;
        sqlx::query("UPDATE gatherings SET first_day = $1, last_day = $2")
            .bind(api::testing::day(-3))
            .bind(api::testing::day(-1))
            .execute(&pool)
            .await
            .unwrap();
        for cancelled in [false, true] {
            if cancelled {
                sqlx::query("UPDATE gatherings SET cancelled_at = now()")
                    .execute(&pool)
                    .await
                    .unwrap();
            }
            assert_eq!(open(&pool, Some(&secret)).await.1["calendar"], false);
            assert_eq!(
                download(&pool, Some(&secret)).await.0,
                StatusCode::NOT_FOUND
            );
        }
    }

    #[tokio::test]
    async fn calendar_without_a_secret() {
        let database = fresh_database().await;
        let pool = database.pool();
        let secret = invited(&pool, 10, 12).await;
        respond(&pool, &secret, "accepted").await;
        for secret in [Some("0".repeat(64)), None] {
            let (status, _, body) = download(&pool, secret.as_deref()).await;
            assert_eq!(status, StatusCode::NOT_FOUND);
            assert_eq!(body, "", "the answer must reveal nothing");
        }
    }

    #[tokio::test]
    async fn request_with_a_personal_link() {
        let logs = crate::test_logs::captured_logs();
        let database = fresh_database().await;
        let pool = database.pool();
        let secret = invited(&pool, 10, 12).await;
        let (status, _) = open(&pool, Some(&secret)).await;
        assert_eq!(status, StatusCode::OK);
        assert!(
            !logs.text().contains(&secret),
            "no log line may contain the secret"
        );
    }
}
