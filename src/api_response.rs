use std::sync::Arc;

use axum::{response::IntoResponse, Json};
use serde::Serialize;
use serde_json::{json, Value};

use crate::AppState;

#[derive(Serialize)]
pub enum JsonResponse {
    Data(DataResponse),
    Error(ErrorResponse),
    Paginate(PaginatedResponse),
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: Value,
    pub message: String,
}

#[derive(Serialize)]
pub struct DataResponse {
    pub data: Value,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse {
    pub data: Value,
    pub _metadata: ResponseMetadata,
    pub message: String,
}

#[derive(Debug, Serialize, Default)]
pub struct ResponseMetadata {
    pub count: u64,
    pub per_page: u64,
    pub total_page: u64,
    pub first_page_url: Option<String>,
    pub last_page_url: Option<String>,
    pub previous_url: Option<String>,
    pub current_url: String,
    pub next_url: Option<String>,
}

impl JsonResponse {
    pub fn error(err: impl Serialize, message: Option<String>) -> JsonResponse {
        let default_message = String::from("An error occured.");

        Self::Error(ErrorResponse {
            error: json!(err),
            message: message.unwrap_or(default_message),
        })
    }
    pub fn data(data: impl Serialize, message: Option<String>) -> JsonResponse {
        let default_message = String::from("Data retrieved successfully.");

        Self::Data(DataResponse {
            data: json!(data),
            message: message.unwrap_or(default_message),
        })
    }
    pub fn paginate(
        data: impl Serialize,
        metadata: ResponseMetadata,
        message: Option<String>,
    ) -> JsonResponse {
        let default_message = String::from("Data retrieved successfully.");

        Self::Paginate(PaginatedResponse {
            _metadata: metadata,
            data: json!(data),
            message: message.unwrap_or(default_message),
        })
    }
}

impl ResponseMetadata {
    pub fn new(ctx: &Arc<AppState>, count: u64, url: String) -> Self {
        let per_page = ctx.config.per_page as u64;

        Self {
            count,
            per_page,
            total_page: count.div_ceil(per_page),
            current_url: url,
            ..Default::default()
        }
    }
}

impl IntoResponse for JsonResponse {
    fn into_response(self) -> axum::response::Response {
        match self {
            JsonResponse::Data(data) => Json(data).into_response(),
            JsonResponse::Error(err) => Json(err).into_response(),
            JsonResponse::Paginate(paginated_response) => Json(paginated_response).into_response(),
        }
    }
}
