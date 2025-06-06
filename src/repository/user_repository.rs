use std::{collections::HashMap, sync::Arc};

use sea_orm::{ColumnTrait, DbErr, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};

use crate::{
    api_response::ResponseMetadata,
    error::AppError,
    models::_entities::{user, user_profile},
    serializer::UserWithProfileSerializer,
    state::AppState,
};

use super::repository_trait::RepositoryTrait;

pub type UserWithProfileModel = (user::Model, Option<user_profile::Model>);

pub struct UserRepository {
    pub app_state: Arc<AppState>,
    pub original_url: Option<String>,
}

impl RepositoryTrait for UserRepository {
    async fn create(
        &self,
        payload: crate::form::user_form::CreateUserRequest,
    ) -> Result<user::Model, AppError> {
        todo!()
    }
}

impl UserRepository {
    pub fn new(app_state: Arc<AppState>, original_url: Option<String>) -> Self {
        Self {
            app_state,
            original_url,
        }
    }

    pub async fn filter_users(
        &self,
        filters: HashMap<String, String>,
    ) -> Result<(Vec<UserWithProfileModel>, ResponseMetadata), AppError> {
        let mut user_query = user::Entity::find().find_also_related(user_profile::Entity);

        if let Some(name) = filters.get("name") {
            user_query = user_query.filter(user::Column::Name.contains(name));
        }

        if let Some(username) = filters.get("username") {
            user_query = user_query.filter(user::Column::Username.contains(username));
        }

        if let Some(email) = filters.get("email") {
            user_query = user_query.filter(user::Column::Email.contains(email));
        }

        let page = filters
            .get("page")
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(1);

        let users_count = user_query.clone().count(&self.app_state.db).await?;

        let response_metadata = ResponseMetadata::new(
            &self.app_state,
            users_count,
            self.original_url.clone().unwrap(),
        );

        let users = user_query
            .order_by(user::Column::DateCreated, sea_orm::Order::Desc)
            .paginate(&self.app_state.db, self.app_state.config.per_page as u64)
            .fetch_page(page - 1)
            .await?;

        // (Vec<UserWithProfileModel>, ResponseMetadata)

        Ok((users, response_metadata))
    }

    pub async fn find_by_id(&self, user_id: i32) -> Result<UserWithProfileModel, AppError> {
        let user_model = user::Entity::find()
            .filter(user::Column::Id.eq(user_id))
            .find_also_related(user_profile::Entity)
            .one(&self.app_state.db)
            .await?
            .ok_or(DbErr::RecordNotFound("User not found".to_string()))?;

        Ok(user_model)
    }

    pub async fn _find_by_username(
        &self,
        username: &str,
    ) -> Result<UserWithProfileModel, AppError> {
        let user_model = user::Entity::find()
            .find_also_related(user_profile::Entity)
            .filter(user::Column::Username.eq(username))
            .one(&self.app_state.db)
            .await?
            .ok_or(DbErr::RecordNotFound("User not found".to_string()))?;

        Ok(user_model)
    }

    // url: /api/users?name__contains=anish&&email__contains=anish
    pub async fn _find_users_by_name(&self, username: &str) -> Result<user::Model, AppError> {
        // cache key = "name__contains=anish&email__contains=anish"
        let user_model = user::Entity::find()
            .filter(user::Column::Username.eq(username))
            .one(&self.app_state.db)
            .await?
            .ok_or(DbErr::RecordNotFound("User not found".to_string()))?;

        Ok(user_model)
    }
}
