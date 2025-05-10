use std::sync::Arc;

use axum::{Json, response::IntoResponse};
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum JsonResponseKind<T: Serialize> {
    Data {
        data: T,
    },
    Error {
        error: serde_json::Value,
    },
    Paginated {
        data: T,
        metadata: ResponseMetadata2,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonResponse2<T: Serialize> {
    #[serde(flatten)]
    kind: JsonResponseKind<T>,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseMetadata2 {
    pub count: u64,
    pub per_page: u64,
    pub total_pages: u64,
    pub current_url: String,
    pub first_page_url: Option<String>,
    pub last_page_url: Option<String>,
    pub previous_url: Option<String>,
    pub next_url: Option<String>,
}

#[derive(Debug)]
pub enum ResponseError {
    Serialization(serde_json::Error),
}

impl<T: Serialize> JsonResponse2<T> {
    pub fn data(data: T) -> Self {
        Self {
            kind: JsonResponseKind::Data { data },
            message: String::from("Data retrieved successfully"),
        }
    }

    pub fn error<E: Serialize>(error: E) -> Result<Self, ResponseError> {
        let error_value = serde_json::to_value(error).map_err(ResponseError::Serialization)?;
        Ok(Self {
            kind: JsonResponseKind::Error { error: error_value },
            message: String::from("An error occurred"),
        })
    }

    pub fn paginated(data: T, metadata: ResponseMetadata2) -> Self {
        Self {
            kind: JsonResponseKind::Paginated { data, metadata },
            message: String::from("Data retrieved successfully"),
        }
    }

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }
}

impl ResponseMetadata2 {
    pub fn new(app_state: &Arc<AppState>, count: u64, current_url: impl Into<String>) -> Self {
        let per_page = app_state.config.per_page as u64;
        let total_pages = count.div_ceil(per_page);

        Self {
            count,
            per_page,
            total_pages,
            current_url: current_url.into(),
            first_page_url: None,
            last_page_url: None,
            previous_url: None,
            next_url: None,
        }
    }

    pub fn with_first_page_url(mut self, url: impl Into<String>) -> Self {
        self.first_page_url = Some(url.into());
        self
    }

    pub fn with_last_page_url(mut self, url: impl Into<String>) -> Self {
        self.last_page_url = Some(url.into());
        self
    }

    pub fn with_previous_url(mut self, url: impl Into<String>) -> Self {
        self.previous_url = Some(url.into());
        self
    }

    pub fn with_next_url(mut self, url: impl Into<String>) -> Self {
        self.next_url = Some(url.into());
        self
    }
}

impl<T: Serialize> IntoResponse for JsonResponse2<T> {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}

impl From<serde_json::Error> for ResponseError {
    fn from(err: serde_json::Error) -> Self {
        ResponseError::Serialization(err)
    }
}

// let response = JsonResponse::data(data).with_message("Custom message");
// let paginated = JsonResponse::paginated(data, metadata).with_message("Paginated data");
// let error = JsonResponse::error(error_data)?;
