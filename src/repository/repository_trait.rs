use crate::{
    error::AppError,
    form::user_form::{CreateUserRequest, UpdateUserRequest},
    models::_entities::user,
};

use super::user_repository::UserWithProfileModel;

pub trait RepositoryTrait: Send + Sync {
    // async fn find_by_id(&self, id: i32) -> Result<Option<user::Model>, AppError>;
    // async fn find_by_email(&self, email: &str) -> Result<Option<user::Model>, AppError>;
    async fn create(&self, payload: CreateUserRequest) -> Result<UserWithProfileModel, AppError>;
    async fn update(&self, id: i32, payload: UpdateUserRequest) -> Result<user::Model, AppError>;
    async fn delete(&self, id: i32) -> Result<(), AppError>;
}
