use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::api_response::JsonResponse;

#[derive(Debug)]
pub enum AppError {
    DatabaseError(sqlx::Error),
    GenericError(String),
    SeaOrm(sea_orm::DbErr),
    Validation(validator::ValidationErrors),
    Unauthorized,
}

impl From<sqlx::Error> for AppError {
    fn from(v: sqlx::Error) -> Self {
        Self::DatabaseError(v)
    }
}

impl From<sea_orm::DbErr> for AppError {
    fn from(v: sea_orm::DbErr) -> Self {
        Self::SeaOrm(v)
    }
}

impl From<validator::ValidationErrors> for AppError {
    fn from(value: validator::ValidationErrors) -> Self {
        Self::Validation(value)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status_code, error_message) = match self {
            AppError::DatabaseError(sqlx_error) => match sqlx_error {
                sqlx::Error::Database(database_error) => {
                    (StatusCode::NOT_FOUND, database_error.to_string())
                }
                sqlx::Error::RowNotFound => (StatusCode::NOT_FOUND, "Row not found".to_string()),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database Error".to_string(),
                ),
            },
            AppError::GenericError(e) => (StatusCode::BAD_REQUEST, e),
            AppError::SeaOrm(db_err) => (StatusCode::NOT_FOUND, db_err.to_string()),
            AppError::Validation(validation_errors) => {
                (StatusCode::BAD_REQUEST, validation_errors.to_string())
            }
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "You are not authorized.".to_string(),
            ),
        };

        (
            status_code,
            JsonResponse::error(error_message, Some("Error".to_string())),
        )
            .into_response()
    }
}
