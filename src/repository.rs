use sea_orm::{
    ActiveModelBehavior, ActiveModelTrait, DatabaseConnection, EntityTrait, IntoActiveModel,
};

use crate::models::_entities::role;

#[async_trait::async_trait]
pub trait Repository<A, E>
where
    E: EntityTrait,
    A: ActiveModelTrait + ActiveModelBehavior + Send + 'static,
    <<A as ActiveModelTrait>::Entity as EntityTrait>::Model: IntoActiveModel<A>,
{
    async fn create(&self, db_connection: &DatabaseConnection, model: A) -> Result<(), String> {
        model.insert(db_connection).await.unwrap();

        Ok(())
    }
}

pub struct RoleRepository;

impl Repository<role::ActiveModel, role::Entity> for RoleRepository {}
