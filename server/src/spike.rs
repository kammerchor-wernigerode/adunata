use axum::Json;
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use time::Date;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum Attendance {
    Accepted,
    Tentative,
    Declined,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SpikeResponse {
    #[schema(min_length = 1, max_length = 100)]
    pub name: String,
    pub attendance: Attendance,
    pub arrival: Option<Date>,
    #[schema(minimum = 0, maximum = 20)]
    pub beds: u8,
}

pub fn router() -> OpenApiRouter {
    OpenApiRouter::new().routes(routes!(create_response))
}

#[utoipa::path(
    post,
    path = "/api/spike-responses",
    request_body = SpikeResponse,
    responses((status = 201, description = "Response stored", body = SpikeResponse))
)]
async fn create_response(Json(response): Json<SpikeResponse>) -> (StatusCode, Json<SpikeResponse>) {
    (StatusCode::CREATED, Json(response))
}
