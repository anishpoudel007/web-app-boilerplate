use axum::{response::IntoResponse, Json};
use serde::Serialize;
use serde_json::{json, Value};

#[derive(Serialize)]
pub enum JsonResponse {
    Error(ErrorResponse),
    Data(DataResponse),
    Paginate(PaginatedResponse),
}

impl JsonResponse {
    pub fn error(err: impl Serialize, message: Option<String>) -> JsonResponse {
        Self::Error(ErrorResponse {
            error: json!(err),
            message: message.or(Some("An error occured.".to_string())),
        })
    }
    pub fn data(data: impl Serialize, message: Option<String>) -> JsonResponse {
        Self::Data(DataResponse {
            data: json!(data),
            message: message.or(Some("Data retrieved successfully".to_string())),
        })
    }
    pub fn paginate(
        data: impl Serialize,
        metadata: ResponseMetadata,
        message: Option<String>,
    ) -> JsonResponse {
        Self::Paginate(PaginatedResponse {
            _metadata: metadata,
            data: json!(data),
            message: message.or(Some("Data retrieved successfully".to_string())),
        })
    }
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: Value,
    pub message: Option<String>,
}

#[derive(Serialize)]
pub struct DataResponse {
    pub data: Value,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Default)]
pub struct ResponseMetadata {
    pub count: u64,
    pub per_page: u64,
    pub total_page: u64,
    pub first_page_url: Option<String>,
    pub last_page_url: Option<String>,
    pub previous_url: Option<String>,
    pub current_url: Option<String>,
    pub next_url: Option<String>,
}

impl ResponseMetadata {
    pub fn new(count: u64, url: Option<String>) -> Self {
        Self {
            count,
            per_page: 10,
            total_page: count.div_ceil(10),
            current_url: url,
            ..Default::default()
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse {
    pub data: Value,
    pub _metadata: ResponseMetadata,
    pub message: Option<String>,
}

// error: [{"title": ["Row not found", "hello"]}, {"email": ["Not valid", ""]}]
//
//

impl IntoResponse for JsonResponse {
    fn into_response(self) -> axum::response::Response {
        match self {
            JsonResponse::Error(err) => Json(err).into_response(),
            JsonResponse::Data(data) => Json(data).into_response(),
            JsonResponse::Paginate(paginated_response) => Json(paginated_response).into_response(),
        }
    }
}
