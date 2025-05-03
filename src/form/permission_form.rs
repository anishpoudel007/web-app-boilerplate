use sea_orm::{DeriveIntoActiveModel, Set};
use serde::Deserialize;

use crate::models::_entities::permission::ActiveModel;

#[derive(Debug, Deserialize, garde::Validate, Clone, DeriveIntoActiveModel)]
pub struct CreatePermissionRequest {
    #[garde(length(min = 5, max = 100))]
    pub name: String,

    #[garde(length(min = 3, max = 50))]
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

#[derive(Debug, Deserialize, garde::Validate, Clone, DeriveIntoActiveModel)]
pub struct UpdatePermissionRequest {
    #[garde(length(min = 5, max = 100))]
    pub name: String,

    #[garde(length(min = 3, max = 50))]
    pub code_name: String,
}
