use crate::{models::_entities::user::ActiveModel, utils::hash};
use sea_orm::Set;

use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, Clone)]
pub struct CreateUserRequest {
    #[validate(length(min = 3, message = "Must have at least 3 characters."))]
    pub name: String,

    #[validate(length(min = 5, message = "Must have at least 5 characters."))]
    pub username: String,

    #[validate(email(message = "Please provide valid email."))]
    pub email: String,

    pub password: String,
    pub address: String,
    pub mobile_number: String,
}

impl From<CreateUserRequest> for ActiveModel {
    fn from(value: CreateUserRequest) -> Self {
        Self {
            name: Set(value.name),
            username: Set(value.username),
            email: Set(value.email),
            password: Set(hash(value.password.as_ref())),
            ..Default::default()
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserRequest {
    #[validate(length(min = 3, message = "Must have at least 3 characters"))]
    pub name: String,

    pub username: String,
    pub email: String,

    #[validate(length(min = 8, message = "Must have at least 8 characters"))]
    pub password: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UserLogin {
    #[validate(length(min = 3, message = "Must have at least 3 characters"))]
    pub username: String,

    #[validate(length(min = 8, message = "Must have at least 8 characters"))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UserRegisterRequest {
    #[validate(length(min = 3, message = "Must have at least 3 characters"))]
    pub username: String,

    #[validate(length(min = 8, message = "Must have at least 8 characters"))]
    pub password: String,
}
