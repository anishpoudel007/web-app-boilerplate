use crate::api_response::JsonResponse;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use sea_orm::error::DbErr;
use serde_json::json;
use std::collections::HashMap;
use validator::ValidationErrors;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    GenericError(String),

    #[error("SeaORM error: {0}")]
    DatabaseError(#[from] DbErr),

    #[error("Validation error: {0}")]
    Validation(#[from] ValidationErrors),

    #[error("Garde validation error: {0}")]
    GardeValidation(#[from] garde::Report),

    #[error("Unauthorized access")]
    Unauthorized,

    #[error("No Token Found.")]
    EmptyToken,

    #[error("Token Expired")]
    InvalidToken,

    #[error("Token Expired")]
    TokenExpired,

    #[error("Forbidden")]
    Forbidden,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status_code, err, message) = match self {
            AppError::GenericError(msg) => (StatusCode::BAD_REQUEST, json!(msg), "Error".into()),
            AppError::DatabaseError(db_err) => match db_err {
                DbErr::RecordNotFound(msg) => {
                    (StatusCode::NOT_FOUND, json!(msg), "Database Error".into())
                }
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!("Database operation failed"),
                    "Database Error".into(),
                ),
            },
            AppError::Validation(errors) => {
                let error_map = format_validation_errors(&errors);
                let error_json = json!(error_map);
                (
                    StatusCode::BAD_REQUEST,
                    error_json,
                    "Validation Error".into(),
                )
            }
            AppError::Forbidden => (
                StatusCode::FORBIDDEN,
                json!("You are not allowed to access the resource"),
                "Forbidden Access".into(),
            ),
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                json!("Unauthorized access"),
                "Authentication Error".into(),
            ),
            AppError::EmptyToken => (
                StatusCode::FORBIDDEN,
                json!("Token Not Found"),
                "Empty Token".into(),
            ),
            AppError::InvalidToken => (
                StatusCode::UNAUTHORIZED,
                json!("Invalid Token"),
                "Authentication Error".into(),
            ),
            AppError::TokenExpired => (
                StatusCode::UNAUTHORIZED,
                json!("Token Expired."),
                "Token Expired.".into(),
            ),
            AppError::GardeValidation(report) => {
                let error_map = format_garde_validation_errors(report);
                let error_json = json!(error_map);
                (
                    StatusCode::BAD_REQUEST,
                    error_json,
                    "Validation Error".into(),
                )
            }
        };

        let payload = JsonResponse::error(err, Some(message));

        (status_code, payload).into_response()
    }
}

/// Formats validation errors from `validator::ValidationErrors` into a structured `HashMap`.
fn format_validation_errors(errors: &ValidationErrors) -> HashMap<String, Vec<String>> {
    tracing::error!("{:#?}", errors);
    errors
        .field_errors()
        .iter()
        .map(|(field, errs)| {
            let messages = errs
                .iter()
                .filter_map(|e| e.message.as_ref().map(|msg| msg.to_string()))
                .collect();
            (field.to_string(), messages)
        })
        .collect()
}

/// Formats validation errors from `garde::Report` into a structured `HashMap`.
fn format_garde_validation_errors(report: garde::Report) -> HashMap<String, Vec<String>> {
    tracing::error!("{:#?}", report);

    report
        .iter()
        .fold(HashMap::new(), |mut acc, (path, error)| {
            let key = path.to_string();
            acc.entry(key).or_default().push(error.to_string());
            acc
        })
}
