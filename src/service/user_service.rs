use std::collections::HashMap;

use crate::{
    api_response::ResponseMetadata,
    error::AppError,
    models::_entities::user,
    repository::user_repository::{UserRepository, UserWithProfileModel},
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
    ) -> Result<(Vec<UserWithProfileModel>, ResponseMetadata), AppError> {
        // convert filters into string to make key for cache
        let user = self.repo.filter_users(filters).await;
        user
    }
}
