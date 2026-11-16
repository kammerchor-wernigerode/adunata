use axum::Json;
use axum::extract::State;
use serde::{Deserialize, Serialize};
use time::Date;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::api::AppState;
use crate::invitations::PersonalLink;

time::serde::format_description!(iso_date, Date, "[year]-[month]-[day]");

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
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(invitation_page))
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
async fn invitation_page(
    State(state): State<AppState>,
    link: PersonalLink,
) -> Json<InvitationPage> {
    let today = state.time_zone.today();
    let gathering = link.gathering;
    let link_state = if gathering.cancelled {
        LinkState::Cancelled
    } else if gathering.ended {
        LinkState::Ended
    } else if gathering.has_begun(today) {
        LinkState::Begun
    } else {
        LinkState::Open
    };
    Json(InvitationPage {
        first_name: link.member.first_name,
        gathering: InvitedGathering {
            title: gathering.title,
            location: gathering.location,
            description: gathering.description,
            first_day: gathering.first_day,
            last_day: gathering.last_day,
        },
        state: link_state,
    })
}

#[cfg(test)]
mod tests {
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode, header};
    use serde_json::{Value, json};
    use sqlx::PgPool;
    use tower::ServiceExt;

    use crate::access::testing::signed_in_organizer;
    use crate::api;
    use crate::database::testing::fresh_database;
    use crate::invitations::testing::{gathering, member, send};

    /// Invites one member to a gathering through the API and returns the secret from the queued mail.
    async fn invited(pool: &PgPool, first: i32, last: i32) -> String {
        let (_, cookie) = signed_in_organizer(pool).await;
        let clara = member(pool, "Clara", "de").await;
        let weekend = gathering(pool, "Probenwochenende", first, last).await;
        sqlx::query("UPDATE gatherings SET first_day = CURRENT_DATE + 10, last_day = CURRENT_DATE + 12 WHERE id = $1")
            .bind(weekend)
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
        sqlx::query("UPDATE gatherings SET first_day = CURRENT_DATE + $2, last_day = CURRENT_DATE + $3 WHERE id = $1")
            .bind(weekend)
            .bind(first)
            .bind(last)
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
    async fn open_on_the_first_day() {
        let database = fresh_database().await;
        let pool = database.pool();
        let secret = invited(&pool, 0, 2).await;
        let (_, page) = open(&pool, Some(&secret)).await;
        assert_eq!(page["state"], "begun");
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
