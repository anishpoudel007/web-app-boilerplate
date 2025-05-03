use crate::{
    models::_entities::user::{self, ActiveModel},
    state::AppState,
    utils::hash,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set};
use std::sync::Arc;

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, garde::Validate)]
#[garde(context(Arc<AppState>))]
pub struct CreateUserRequest {
    #[garde(length(min = 3, max = 100))]
    pub name: String,

    #[garde(length(min = 5, max = 100))]
    #[garde(custom(CreateUserRequest::validate_username_exists))]
    pub username: String,

    #[garde(email)]
    #[garde(custom(CreateUserRequest::validate_email_exists))]
    pub email: String,

    #[garde(length(min = 8))]
    pub password: String,

    #[garde(length(max = 200))]
    pub address: String,

    #[garde(length(max = 50))]
    pub mobile_number: String,
}

impl CreateUserRequest {
    fn validate_username_exists(value: &str, context: &Arc<AppState>) -> garde::Result {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                match user::Entity::find()
                    .filter(user::Column::Username.eq(value))
                    .one(&context.db)
                    .await
                {
                    Ok(Some(_)) => Err(garde::Error::new(
                        "User with the given username already exists.",
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
    fn validate_email_exists(value: &str, context: &Arc<AppState>) -> garde::Result {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                match user::Entity::find()
                    .filter(user::Column::Email.eq(value))
                    .one(&context.db)
                    .await
                {
                    Ok(Some(_)) => Err(garde::Error::new(
                        "User with the given email already exists.",
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

#[derive(Debug, Deserialize, garde::Validate)]
pub struct UpdateUserRequest {
    #[garde(length(min = 3, max = 100))]
    pub name: String,

    #[garde(length(min = 3, max = 100))]
    pub username: String,

    #[garde(email)]
    #[garde(length(min = 3, max = 100))]
    pub email: String,

    #[garde(length(min = 8, max = 100))]
    pub password: Option<String>,
}

#[derive(Debug, Deserialize, garde::Validate)]
pub struct UserLogin {
    #[garde(length(min = 3, max = 100))]
    pub username: String,

    #[garde(length(min = 8, max = 100))]
    pub password: String,
}

#[derive(Debug, Deserialize, garde::Validate)]
pub struct UserRegisterRequest {
    #[garde(length(min = 3, max = 100))]
    pub username: String,

    #[garde(length(min = 8, max = 100))]
    pub password: String,
}
