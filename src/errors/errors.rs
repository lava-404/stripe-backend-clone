use axum::{Json, http::StatusCode, response::{IntoResponse, Response}};
use serde::Serialize;

#[derive(Debug)]
pub enum AppError {
    UserNotFound,
    InvalidCredentials,
    EmailAlreadyExists,
    DatabaseError,
}

#[derive(Serialize)]
struct ErrorResponse {
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::UserNotFound => (StatusCode::NOT_FOUND, "User does not exist"),
            AppError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "Invalid email or password"),
            AppError::EmailAlreadyExists => (StatusCode::CONFLICT, "Email already exists"),
            AppError::DatabaseError => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
        };

        (
            status,
            Json(ErrorResponse {
                message: message.to_string(),
            }),
        )
            .into_response()
    }
}