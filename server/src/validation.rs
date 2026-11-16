use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Why a field was rejected. The client turns each code into a message from the catalog.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum ErrorCode {
    Required,
    TooLong,
    TooLarge,
    Taken,
    Invalid,
    LastOrganizer,
    BeforeFirstDay,
    Ended,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct FieldError {
    pub field: String,
    pub code: ErrorCode,
    /// The stored value that a taken value collides with, such as a mail address that differs only in case.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conflict: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ValidationErrors {
    pub errors: Vec<FieldError>,
}

pub fn field_error(field: &str, code: ErrorCode) -> FieldError {
    FieldError {
        field: field.to_owned(),
        code,
        conflict: None,
    }
}

/// Answers with status 422 and the errors of each field.
pub fn invalid(errors: Vec<FieldError>) -> Response {
    (
        StatusCode::UNPROCESSABLE_ENTITY,
        Json(ValidationErrors { errors }),
    )
        .into_response()
}

pub fn is_unique_violation(error: &sqlx::Error) -> bool {
    error
        .as_database_error()
        .is_some_and(sqlx::error::DatabaseError::is_unique_violation)
}
