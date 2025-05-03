use std::sync::Arc;

use sea_orm::{ColumnTrait, DeriveIntoActiveModel, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use tokio::runtime::Handle;

use crate::{
    models::_entities::role::{self, ActiveModel},
    state::AppState,
};

#[derive(Debug, Deserialize, garde::Validate, DeriveIntoActiveModel)]
#[garde(context(Arc<AppState>))]
pub struct CreateRoleRequest {
    #[garde(length(min = 3, max = 100))]
    #[garde(custom(CreateRoleRequest::validate_role_exists))]
    pub name: String,
}

impl CreateRoleRequest {
    fn validate_role_exists(value: &str, context: &Arc<AppState>) -> garde::Result {
        tokio::task::block_in_place(|| {
            Handle::current().block_on(async {
                match role::Entity::find()
                    .filter(role::Column::Name.eq(value))
                    .one(&context.db)
                    .await
                {
                    Ok(Some(_)) => Err(garde::Error::new(
                        "Role with the given name already exists.",
                    )),

                    Ok(None) => Ok(()),

                    Err(e) => {
                        tracing::error!("Database error during role validation: {:?}", e);

                        Err(garde::Error::new("Internal error during validation."))
                    }
                }
            })
        })
    }
}

impl From<CreateRoleRequest> for ActiveModel {
    fn from(value: CreateRoleRequest) -> Self {
        Self {
            name: Set(value.name),
            ..Default::default()
        }
    }
}

#[derive(Debug, Deserialize, garde::Validate, DeriveIntoActiveModel)]
pub struct UpdateRoleRequest {
    #[garde(length(min = 3, max = 100))]
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, garde::Validate)]
pub struct UpdateUserRolesRequest {
    #[garde(skip)]
    pub roles: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, garde::Validate)]
pub struct UpdateUserPermissionRequest {
    #[garde(skip)]
    pub permissions: Vec<String>,
}
