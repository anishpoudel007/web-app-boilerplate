use sea_orm::{DeriveIntoActiveModel, Set};
use serde::Deserialize;
use validator::Validate;

use crate::models::_entities::permission::ActiveModel;

#[derive(Debug, Deserialize, Validate, Clone, DeriveIntoActiveModel)]
pub struct CreatePermissionRequest {
    #[validate(length(min = 3, message = "Must have at least 3 characters"))]
    pub name: String,
    pub code_name: String,
}

impl From<CreatePermissionRequest> for ActiveModel {
    fn from(value: CreatePermissionRequest) -> Self {
        Self {
            name: Set(value.name),
            code_name: Set(value.code_name),
            ..Default::default()
        }
    }
}

#[derive(Debug, Deserialize, Validate, Clone, DeriveIntoActiveModel)]
pub struct UpdatePermissionRequest {
    #[validate(length(min = 3, message = "Must have at least 3 characters"))]
    pub name: String,
    pub code_name: String,
}
