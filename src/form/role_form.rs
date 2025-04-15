use sea_orm::{DeriveIntoActiveModel, Set};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::models::_entities::role::ActiveModel;

#[derive(Debug, Deserialize, Validate, DeriveIntoActiveModel, Clone)]
pub struct CreateRoleRequest {
    #[validate(length(min = 3, message = "Must have at least 3 characters"))]
    pub name: String,
}

impl From<CreateRoleRequest> for ActiveModel {
    fn from(value: CreateRoleRequest) -> Self {
        Self {
            name: Set(value.name),
            ..Default::default()
        }
    }
}

#[derive(Debug, Deserialize, Validate, DeriveIntoActiveModel)]
pub struct UpdateRoleRequest {
    #[validate(length(min = 3, message = "Must have at least 3 characters"))]
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateUserRolesRequest {
    pub roles: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateUserPermissionRequest {
    pub permissions: Vec<String>,
}
