use std::collections::HashMap;

use garde::Validate as _;
use sea_orm::{
    ActiveModelTrait as _,
    ActiveValue::{NotSet, Set},
    ColumnTrait as _, Condition, DbErr, EntityTrait as _, IntoActiveModel, QueryFilter as _,
    TransactionTrait as _,
};

use crate::{
    api_response::ResponseMetadata,
    error::AppError,
    form::user_form::{CreateUserRequest, UpdateUserRequest},
    json_response::ResponseMetadata2,
    models::_entities::{user, user_profile},
    repository::{
        repository_trait::RepositoryTrait,
        user_repository::{UserRepository, UserWithProfileModel},
    },
    serializer::UserSerializer,
    utils::hash,
};

use super::service_trait::ServiceTrait;

pub struct UserService<'a> {
    repo: &'a UserRepository,
}

impl<'a> UserService<'a> {
    pub fn new(repo: &'a UserRepository) -> Self {
        Self { repo }
    }
}

impl ServiceTrait for UserService<'_> {
    async fn get_user(&self, id: i32) -> Result<UserWithProfileModel, AppError> {
        // id can also be used as cache key ??
        let user = self.repo.find_by_id(id).await;
        user
    }
    async fn get_users(
        &self,
        filters: HashMap<String, String>,
    ) -> Result<(Vec<UserWithProfileModel>, ResponseMetadata2), AppError> {
        // convert filters into string to make key for cache
        let user = self.repo.filter_users(filters).await;
        user
    }

    async fn create_user(
        &self,
        payload: CreateUserRequest,
    ) -> Result<UserWithProfileModel, AppError> {
        payload.validate_with(&self.repo.app_state)?;

        let user_exist = user::Entity::find()
            .filter(
                Condition::any()
                    .add(user::Column::Email.eq(&payload.email))
                    .add(user::Column::Username.eq(&payload.username)),
            )
            .one(&self.repo.app_state.db)
            .await?;

        if user_exist.is_some() {
            return Err(AppError::GenericError(
                "A user with this email or username already exists.".to_string(),
            ));
        }

        let user_with_profile = self.repo.create(payload).await?;

        Ok(user_with_profile)
    }

    async fn update_user(
        &self,
        id: i32,
        payload: UpdateUserRequest,
    ) -> Result<user::Model, AppError> {
        let user = user::Entity::find_by_id(id)
            .one(&self.repo.app_state.db)
            .await?
            .ok_or(DbErr::RecordNotFound("User not found.".to_string()))?;

        payload.validate()?;

        let mut user = user.into_active_model();

        user.name = Set(payload.name);
        user.username = Set(payload.username);
        user.email = Set(payload.email);
        user.password = payload.password.map_or_else(|| NotSet, |x| Set(hash(&x)));

        let updated_user = user.update(&self.repo.app_state.db).await?;

        Ok(updated_user)
    }

    async fn delete_user(&self, id: i32) -> Result<(), AppError> {
        let _ = user::Entity::find_by_id(id)
            .one(&self.repo.app_state.db)
            .await?
            .ok_or(DbErr::RecordNotFound("User not found.".to_string()))?;

        Ok(())
    }
}
