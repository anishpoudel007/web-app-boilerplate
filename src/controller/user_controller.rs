use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use axum::routing::delete;
use axum::Extension;
use axum::{
    extract::{OriginalUri, Path, Query, State},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use sea_orm::Condition;
use sea_orm::{
    ActiveModelTrait, ActiveValue::NotSet, ColumnTrait, DbErr, EntityTrait, ModelTrait,
    PaginatorTrait, QueryFilter, QueryOrder, Set, TransactionTrait,
};
use validator::Validate;

use crate::api_response::{JsonResponse, ResponseMetadata};
use crate::auth::auth_service::AuthService;
use crate::error::AppError;
use crate::extractor::ValidJson;
use crate::form::{
    role_form::{UpdateUserPermissionRequest, UpdateUserRolesRequest},
    user_form::{CreateUserRequest, UpdateUserRequest},
};
use crate::models::_entities::{permission, role, user, user_permission, user_profile, user_role};
use crate::serializer::{
    PermissionSerializer, RoleSerializer, UserSerializer, UserWithProfileSerializer,
};
use crate::AppState;

pub async fn get_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(get_users).post(create_user))
        .route(
            "/{user_id}",
            get(get_user).put(update_user).delete(delete_user),
        )
        .route("/{user_id}/roles", get(get_user_roles).post(assign_roles))
        .route("/{user_id}/roles/sync", post(sync_roles))
        .route("/{user_id}/roles/{role_id}", delete(delete_role))
        .route(
            "/{user_id}/permissions",
            get(get_user_permissions).post(assign_permissions),
        )
        .route("/{user_id}/permissions/sync", post(sync_permissions))
}

#[axum::debug_handler()]
pub async fn get_users(
    State(app_state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
    OriginalUri(original_uri): OriginalUri,
    Extension(user_model): Extension<user::Model>,
) -> Result<impl IntoResponse, AppError> {
    AuthService::has_permission(&app_state, &user_model, "read_users").await?;

    let mut user_query = user::Entity::find().find_also_related(user_profile::Entity);

    if let Some(name) = params.get("name") {
        user_query = user_query.filter(user::Column::Name.contains(name));
    }

    if let Some(username) = params.get("username") {
        user_query = user_query.filter(user::Column::Username.contains(username));
    }

    if let Some(email) = params.get("email") {
        user_query = user_query.filter(user::Column::Email.contains(email));
    }

    let page = params
        .get("page")
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(1);

    let users_count = user_query.clone().count(&app_state.db).await?;

    let response_metadata =
        ResponseMetadata::new(&app_state, users_count, original_uri.to_string());

    let users: Vec<UserWithProfileSerializer> = user_query
        .order_by(user::Column::DateCreated, sea_orm::Order::Desc)
        .paginate(&app_state.db, app_state.config.per_page as u64)
        .fetch_page(page - 1)
        .await?
        .into_iter()
        .map(UserWithProfileSerializer::from)
        .collect();

    Ok(JsonResponse::paginate(users, response_metadata, None))
}

#[axum::debug_handler()]
pub async fn get_user(
    State(app_state): State<Arc<AppState>>,
    Path(user_id): Path<i32>,
    Extension(user_model): Extension<user::Model>,
) -> Result<impl IntoResponse, AppError> {
    AuthService::has_permission(&app_state, &user_model, "read_user").await?;

    let user: UserWithProfileSerializer = user::Entity::find_by_id(user_id)
        .find_also_related(user_profile::Entity)
        .one(&app_state.db)
        .await?
        .ok_or(DbErr::RecordNotFound("User not found.".to_string()))?
        .into();

    Ok(JsonResponse::data(user, None))
}

#[axum::debug_handler]
pub async fn create_user(
    State(app_state): State<Arc<AppState>>,
    Extension(user_model): Extension<user::Model>,
    ValidJson(payload): ValidJson<CreateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    AuthService::has_permission(&app_state, &user_model, "create_user").await?;

    payload.validate()?;

    let user_exist = user::Entity::find()
        .filter(
            Condition::any()
                .add(user::Column::Email.eq(&payload.email))
                .add(user::Column::Username.eq(&payload.username)),
        )
        .one(&app_state.db)
        .await?;

    if user_exist.is_some() {
        return Err(AppError::GenericError(
            "A user with this email or username already exists.".to_string(),
        ));
    }

    let user_with_profile = app_state
        .db
        .transaction::<_, (user::Model, Option<user_profile::Model>), DbErr>(|txn| {
            Box::pin(async move {
                let user = user::ActiveModel::from(payload.clone()).insert(txn).await?;

                let user_profile = user_profile::ActiveModel {
                    id: sea_orm::ActiveValue::NotSet,
                    user_id: Set(user.id),
                    address: Set(Some(payload.address)),
                    mobile_number: Set(Some(payload.mobile_number)),
                }
                .insert(txn)
                .await?;

                Ok((user, Some(user_profile)))
            })
        })
        .await
        .map_err(|e| AppError::GenericError(e.to_string()))?; // should be database error

    let user_serializer = UserWithProfileSerializer::from(user_with_profile);

    Ok(JsonResponse::data(user_serializer, None))
}

#[axum::debug_handler()]
pub async fn update_user(
    State(app_state): State<Arc<AppState>>,
    Path(user_id): Path<i32>,
    Extension(user_model): Extension<user::Model>,
    ValidJson(payload): ValidJson<UpdateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    AuthService::has_permission(&app_state, &user_model, "update_user").await?;

    let user = user::Entity::find_by_id(user_id)
        .one(&app_state.db)
        .await?
        .ok_or(DbErr::RecordNotFound("User not found.".to_string()))?;

    payload.validate()?;

    let mut user: user::ActiveModel = user.into();

    let password = match payload.password {
        Some(pwd) => Set(pwd),
        None => NotSet,
    };

    user.name = Set(payload.name);
    user.username = Set(payload.username);
    user.email = Set(payload.email);
    user.password = password;

    let user_serializer: UserSerializer = user.update(&app_state.db).await?.into();

    Ok(JsonResponse::data(user_serializer, None))
}

#[axum::debug_handler()]
pub async fn delete_user(
    State(app_state): State<Arc<AppState>>,
    Path(user_id): Path<i32>,
    Extension(user_model): Extension<user::Model>,
) -> Result<impl IntoResponse, AppError> {
    AuthService::has_permission(&app_state, &user_model, "delete_user").await?;

    let res = user::Entity::delete_by_id(user_id)
        .exec(&app_state.db)
        .await?;

    println!("{:?}", res);

    Ok(JsonResponse::data(
        None::<String>,
        Some("User deleted successfully".to_string()),
    ))
}

#[axum::debug_handler()]
pub async fn get_user_roles(
    State(app_state): State<Arc<AppState>>,
    Path(user_id): Path<i32>,
    Extension(user_model): Extension<user::Model>,
) -> Result<impl IntoResponse, AppError> {
    AuthService::has_permission(&app_state, &user_model, "read_user_roles").await?;

    let user_with_roles = user::Entity::find_by_id(user_id)
        .find_with_related(role::Entity)
        .all(&app_state.db)
        .await?;

    let role_serializer: Vec<RoleSerializer> = user_with_roles
        .into_iter()
        .flat_map(|(_, role)| role)
        .map(RoleSerializer::from)
        .collect();

    Ok(JsonResponse::data(role_serializer, None))
}

#[axum::debug_handler()]
pub async fn assign_roles(
    State(app_state): State<Arc<AppState>>,
    Path(user_id): Path<i32>,
    Extension(user_model): Extension<user::Model>,
    ValidJson(payload): ValidJson<UpdateUserRolesRequest>,
) -> Result<impl IntoResponse, AppError> {
    AuthService::has_permission(&app_state, &user_model, "assign_roles").await?;

    let _user_model = user::Entity::find_by_id(user_id)
        .one(&app_state.db)
        .await?
        .ok_or(DbErr::RecordNotFound("User not found.".to_string()))?;

    if payload.roles.is_empty() {
        return Err(AppError::GenericError("Empty roles".to_string()));
    }

    let existing_roles: HashSet<String> = user::Entity::find_by_id(user_id)
        .find_with_related(role::Entity)
        .filter(role::Column::Name.is_in(&payload.roles))
        .all(&app_state.db)
        .await?
        .into_iter()
        .flat_map(|(_, roles)| roles.into_iter().map(|role| role.name))
        .collect();

    let requested_roles: HashSet<String> = payload.roles.into_iter().collect();
    let roles_to_add: Vec<String> = requested_roles
        .difference(&existing_roles)
        .cloned()
        .collect();

    if roles_to_add.is_empty() {
        return Ok(JsonResponse::data(
            None::<String>,
            Some("All roles already assigned.".to_string()),
        ));
    }

    // Fetch role details for the new roles
    let roles_to_add_models = role::Entity::find()
        .filter(role::Column::Name.is_in(roles_to_add.clone()))
        .all(&app_state.db)
        .await?;

    // Prepare user_role ActiveModels for insertion
    let user_roles: Vec<user_role::ActiveModel> = roles_to_add_models
        .into_iter()
        .map(|role| user_role::ActiveModel {
            id: NotSet,
            user_id: Set(user_id),
            role_id: Set(role.id),
        })
        .collect();

    if !user_roles.is_empty() {
        user_role::Entity::insert_many(user_roles)
            .exec(&app_state.db)
            .await?;
    }

    Ok(JsonResponse::data(
        roles_to_add,
        Some("Roles added successfully.".to_string()),
    ))
}

#[axum::debug_handler()]
pub async fn get_user_permissions(
    State(app_state): State<Arc<AppState>>,
    Path(user_id): Path<i32>,
    Extension(user_model): Extension<user::Model>,
) -> Result<impl IntoResponse, AppError> {
    AuthService::has_permission(&app_state, &user_model, "read_user_permissions").await?;

    let user_with_permissions = user::Entity::find_by_id(user_id)
        .find_with_related(permission::Entity)
        .all(&app_state.db)
        .await?;

    let permission_serializer: Vec<PermissionSerializer> = user_with_permissions
        .into_iter()
        .flat_map(|(_, permission)| permission)
        .map(PermissionSerializer::from)
        .collect();

    Ok(JsonResponse::data(permission_serializer, None))
}

#[axum::debug_handler()]
pub async fn assign_permissions(
    State(app_state): State<Arc<AppState>>,
    Path(user_id): Path<i32>,
    Extension(user_model): Extension<user::Model>,
    ValidJson(payload): ValidJson<UpdateUserPermissionRequest>,
) -> Result<impl IntoResponse, AppError> {
    AuthService::has_permission(&app_state, &user_model, "assign_permissions").await?;

    let user_model = user::Entity::find_by_id(user_id)
        .one(&app_state.db)
        .await?
        .ok_or(DbErr::RecordNotFound("User not found.".to_string()))?;

    if payload.permissions.is_empty() {
        return Err(AppError::GenericError("Empty permission.".to_string()));
    }

    // retrieve user-related permissions from the database
    let user_permissions: Vec<String> = user_model
        .find_related(permission::Entity)
        .filter(permission::Column::CodeName.is_in(&payload.permissions))
        .all(&app_state.db)
        .await?
        .into_iter()
        .map(|permission| permission.name)
        .collect();

    let permissions_to_add: Vec<String> = payload
        .permissions
        .into_iter()
        .filter(|permission| !user_permissions.contains(permission))
        .collect();

    if permissions_to_add.is_empty() {
        return Ok(JsonResponse::data(
            None::<String>,
            Some("Already added.".to_string()),
        ));
    }

    let new_permissions = permission::Entity::find()
        .filter(permission::Column::CodeName.is_in(permissions_to_add))
        .all(&app_state.db)
        .await?;

    let user_permissions: Vec<user_permission::ActiveModel> = new_permissions
        .iter()
        .map(|permission| user_permission::ActiveModel {
            id: NotSet,
            user_id: Set(user_id),
            permission_id: Set(permission.id),
        })
        .collect();

    if !user_permissions.is_empty() {
        user_permission::Entity::insert_many(user_permissions)
            .exec(&app_state.db)
            .await?;
    }

    Ok(JsonResponse::data(
        new_permissions,
        Some("Roles added successfully".to_string()),
    ))
}

#[axum::debug_handler()]
pub async fn sync_permissions(
    State(app_state): State<Arc<AppState>>,
    Path(user_id): Path<i32>,
    Extension(user_model): Extension<user::Model>,
    ValidJson(payload): ValidJson<UpdateUserPermissionRequest>,
) -> Result<impl IntoResponse, AppError> {
    AuthService::has_permission(&app_state, &user_model, "sync_permissions").await?;

    let _user_model = user::Entity::find_by_id(user_id)
        .one(&app_state.db)
        .await?
        .ok_or(DbErr::RecordNotFound("User not found.".to_string()))?;

    let valid_permissions: HashSet<String> = permission::Entity::find()
        .filter(permission::Column::CodeName.is_in(&payload.permissions))
        .all(&app_state.db)
        .await?
        .into_iter()
        .map(|permission| permission.code_name)
        .collect();

    if valid_permissions.is_empty() {
        // delete all permissions of the user
        let _res = user_permission::Entity::delete_many()
            .filter(user_permission::Column::UserId.eq(user_id))
            .exec(&app_state.db)
            .await?;

        return Ok(JsonResponse::data(
            None::<String>,
            Some("Permission synced successfully.".to_string()),
        ));
    }

    // in database
    // delete others except below
    let user_permissions: HashSet<String> = user::Entity::find_by_id(user_id)
        .find_with_related(permission::Entity)
        .filter(permission::Column::CodeName.is_in(&valid_permissions))
        .all(&app_state.db)
        .await?
        .into_iter()
        .flat_map(|(_, permissions)| permissions.into_iter().map(|value| value.code_name))
        .collect();

    let permissions_to_add: Vec<String> = valid_permissions
        .difference(&user_permissions)
        .cloned()
        .collect();

    let permissions_to_delete: Vec<i32> = user::Entity::find_by_id(user_id)
        .find_with_related(permission::Entity)
        .filter(permission::Column::CodeName.is_not_in(&valid_permissions))
        .all(&app_state.db)
        .await?
        .iter()
        .flat_map(|(_, permissions)| permissions.iter().map(|perm| perm.id))
        .collect();

    if permissions_to_add.is_empty() && permissions_to_delete.is_empty() {
        return Ok(JsonResponse::data(
            None::<String>,
            Some("No changes needed.".to_string()),
        ));
    }

    // Prepare permissions to insert
    let new_permissions = permission::Entity::find()
        .filter(permission::Column::CodeName.is_in(permissions_to_add))
        .all(&app_state.db)
        .await?;

    let new_user_permissions: Vec<user_permission::ActiveModel> = new_permissions
        .iter()
        .map(|permission| user_permission::ActiveModel {
            id: NotSet,
            user_id: Set(user_id),
            permission_id: Set(permission.id),
        })
        .collect();

    app_state
        .db
        .transaction::<_, (), DbErr>(|txn| {
            Box::pin(async move {
                if !new_user_permissions.is_empty() {
                    user_permission::Entity::insert_many(new_user_permissions)
                        .exec(txn)
                        .await?;
                }

                if !permissions_to_delete.is_empty() {
                    user_permission::Entity::delete_many()
                        .filter(user_permission::Column::UserId.eq(user_id))
                        .filter(user_permission::Column::PermissionId.is_in(permissions_to_delete))
                        .exec(txn)
                        .await?;
                }

                Ok(())
            })
        })
        .await
        .map_err(|e| AppError::GenericError(e.to_string()))?; // should be database error

    Ok(JsonResponse::data(
        None::<String>,
        Some("Permissions sync successfully".to_string()),
    ))
}

#[axum::debug_handler()]
pub async fn sync_roles(
    State(app_state): State<Arc<AppState>>,
    Path(user_id): Path<i32>,
    Extension(user_model): Extension<user::Model>,
    ValidJson(payload): ValidJson<UpdateUserRolesRequest>,
) -> Result<impl IntoResponse, AppError> {
    AuthService::has_permission(&app_state, &user_model, "sync_roles").await?;

    let _user_model = user::Entity::find_by_id(user_id)
        .one(&app_state.db)
        .await?
        .ok_or(DbErr::RecordNotFound("User not found.".to_string()))?;

    let valid_roles: HashSet<String> = role::Entity::find()
        .filter(role::Column::Name.is_in(&payload.roles))
        .all(&app_state.db)
        .await?
        .into_iter()
        .map(|role| role.name)
        .collect();

    if valid_roles.is_empty() {
        // delete all roles of the user
        let _res = user_role::Entity::delete_many()
            .filter(user_role::Column::UserId.eq(user_id))
            .exec(&app_state.db)
            .await?;

        return Ok(JsonResponse::data(
            None::<String>,
            Some("Roles synced successfully.".to_string()),
        ));
    }

    // in database
    // delete others except below
    let user_roles: HashSet<String> = user::Entity::find_by_id(user_id)
        .find_with_related(role::Entity)
        .filter(role::Column::Name.is_in(&valid_roles))
        .all(&app_state.db)
        .await?
        .into_iter()
        .flat_map(|(_, roles)| roles.into_iter().map(|role| role.name))
        .collect();

    let roles_to_add: Vec<String> = valid_roles.difference(&user_roles).cloned().collect();

    let roles_to_delete: Vec<i32> = user::Entity::find_by_id(user_id)
        .find_with_related(role::Entity)
        .filter(role::Column::Name.is_not_in(&valid_roles))
        .all(&app_state.db)
        .await?
        .iter()
        .flat_map(|(_, roles)| roles.iter().map(|role| role.id))
        .collect();

    if roles_to_add.is_empty() && roles_to_delete.is_empty() {
        return Ok(JsonResponse::data(
            None::<String>,
            Some("No changes needed.".to_string()),
        ));
    }

    // Prepare roles to insert
    let new_roles = role::Entity::find()
        .filter(role::Column::Name.is_in(roles_to_add))
        .all(&app_state.db)
        .await?;

    let new_user_roles: Vec<user_role::ActiveModel> = new_roles
        .iter()
        .map(|role| user_role::ActiveModel {
            id: NotSet,
            user_id: Set(user_id),
            role_id: Set(role.id),
        })
        .collect();

    app_state
        .db
        .transaction::<_, (), DbErr>(|txn| {
            Box::pin(async move {
                if !new_user_roles.is_empty() {
                    user_role::Entity::insert_many(new_user_roles)
                        .exec(txn)
                        .await?;
                }

                if !roles_to_delete.is_empty() {
                    user_role::Entity::delete_many()
                        .filter(user_role::Column::UserId.eq(user_id))
                        .filter(user_role::Column::RoleId.is_in(roles_to_delete))
                        .exec(txn)
                        .await?;
                }

                Ok(())
            })
        })
        .await
        .map_err(|e| AppError::GenericError(e.to_string()))?; // should be database error

    Ok(JsonResponse::data(
        None::<String>,
        Some("Roles sync successfully".to_string()),
    ))
}

#[axum::debug_handler()]
pub async fn delete_role(
    State(app_state): State<Arc<AppState>>,
    Path((user_id, role_id)): Path<(i32, i32)>,
    Extension(user_model): Extension<user::Model>,
) -> Result<impl IntoResponse, AppError> {
    AuthService::has_permission(&app_state, &user_model, "delete_user_role").await?;

    let _user_model = user::Entity::find_by_id(user_id)
        .one(&app_state.db)
        .await?
        .ok_or(AppError::GenericError("User not found.".to_string()))?;

    let _role = role::Entity::find_by_id(role_id)
        .one(&app_state.db)
        .await?
        .ok_or(AppError::GenericError("Role not found.".to_string()))?;

    let res = user_role::Entity::delete_many()
        .filter(user_role::Column::UserId.eq(user_id))
        .filter(user_role::Column::RoleId.eq(role_id))
        .exec(&app_state.db)
        .await?;

    println!("{:?}", res);

    Ok(JsonResponse::data(
        None::<String>,
        Some("Role removed from the user".to_string()),
    ))
}
