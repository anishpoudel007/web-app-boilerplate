use std::collections::HashMap;

use crate::{
    api_response::ResponseMetadata, error::AppError, form::user_form::CreateUserRequest,
    models::_entities::user, repository::user_repository::UserWithProfileModel,
};

pub trait ServiceTrait: Send + Sync {
    async fn get_user(&self, id: i32) -> Result<UserWithProfileModel, AppError>;
    async fn get_users(
        &self,
        filters: HashMap<String, String>,
    ) -> Result<(Vec<UserWithProfileModel>, ResponseMetadata), AppError>;
    // async fn get_user_by_email(&self, email: &str) -> Result<user::Model, AppError>;
    // async fn create_user(&self, payload: CreateUserRequest) -> Result<user::Model, AppError>;
    // async fn update_user(
    //     &self,
    //     id: i32,
    //     payload: CreateUserRequest,
    // ) -> Result<user::Model, AppError>;
    // async fn delete_user(&self, id: i32) -> Result<(), AppError>;
}
