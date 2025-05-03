use serde::Serialize;

use crate::{
    models::_entities::{permission, role, user, user_profile},
    repository::user_repository::UserWithProfileModel,
};

#[derive(Debug, Serialize)]
pub struct UserSerializer {
    pub id: i32,
    pub name: String,
    pub username: String,
    pub email: String,
}

impl From<user::Model> for UserSerializer {
    fn from(value: user::Model) -> Self {
        Self {
            id: value.id,
            name: value.name,
            username: value.username,
            email: value.email,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct UserProfileSerializer {
    pub id: i32,
    pub address: Option<String>,
    pub mobile_number: Option<String>,
}

impl From<user_profile::Model> for UserProfileSerializer {
    fn from(value: user_profile::Model) -> Self {
        Self {
            id: value.id,
            address: value.address,
            mobile_number: value.mobile_number,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct UserWithProfileSerializer {
    pub id: i32,
    pub name: String,
    pub username: String,
    pub email: String,
    pub profile: Option<UserProfileSerializer>,
}

impl From<UserWithProfileModel> for UserWithProfileSerializer {
    fn from(value: UserWithProfileModel) -> Self {
        let (user, profile) = value;

        let profile_serializer = profile.map(UserProfileSerializer::from);

        Self {
            id: user.id,
            name: user.name,
            username: user.username,
            email: user.email,
            profile: profile_serializer,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PermissionSerializer {
    pub id: i32,
    pub name: String,
    pub code_name: String,
}

impl From<permission::Model> for PermissionSerializer {
    fn from(value: permission::Model) -> Self {
        Self {
            id: value.id,
            name: value.name,
            code_name: value.code_name,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct RoleSerializer {
    pub id: i32,
    pub name: String,
}

impl From<role::Model> for RoleSerializer {
    fn from(value: role::Model) -> Self {
        Self {
            id: value.id,
            name: value.name,
        }
    }
}
