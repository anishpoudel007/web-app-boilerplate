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

    #[error("Unauthorized access")]
    Unauthorized,
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

            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                json!("Unauthorized access"),
                "Authentication Error".into(),
            ),
        };

        let payload = JsonResponse::error(err, Some(message));

        (status_code, payload).into_response()
    }
}

/// Formats validation errors from `validator::ValidationErrors` into a structured `HashMap`.
fn format_validation_errors(errors: &ValidationErrors) -> HashMap<String, Vec<String>> {
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
