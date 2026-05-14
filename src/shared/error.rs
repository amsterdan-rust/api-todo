use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

use crate::todo::error::TodoError;

#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    Validation(String),
    Conflict(String),
    Internal(String),
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NotFound(message) => (StatusCode::NOT_FOUND, message),
            AppError::Validation(message) => (StatusCode::BAD_REQUEST, message),
            AppError::Conflict(message) => (StatusCode::CONFLICT, message),
            AppError::Internal(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
        };

        let body = Json(ErrorResponse { message });

        (status, body).into_response()
    }
}

impl From<TodoError> for AppError {
    fn from(error: TodoError) -> Self {
        let message = error.to_string();

        match error {
            TodoError::NotFound => AppError::NotFound(message),

            TodoError::InvalidTitle | TodoError::TitleTooLong => AppError::Validation(message),

            TodoError::AlreadyCompleted | TodoError::AlreadyOpen => AppError::Conflict(message),

            TodoError::Repository => AppError::Internal(message),
        }
    }
}
