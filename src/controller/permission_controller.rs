use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::get,
    Extension, Json, Router,
};
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel, Set};
use validator::Validate;

use crate::{
    api_response::JsonResponse,
    auth::auth_service::AuthService,
    error::AppError,
    form::permission_form::CreatePermissionRequest,
    models::_entities::{permission, user},
    serializer::PermissionSerializer,
    AppState,
};

pub async fn get_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(get_permissions).post(create_permission))
        .route(
            "/{permission_id}",
            get(get_permission)
                .put(update_permission)
                .delete(delete_permission),
        )
}

#[axum::debug_handler]
pub async fn get_permissions(
    State(app_state): State<Arc<AppState>>,
    Extension(user_model): Extension<user::Model>,
) -> Result<impl IntoResponse, AppError> {
    AuthService::has_permission(&app_state, &user_model, "read_permissions").await?;

    let permissions: Vec<PermissionSerializer> = permission::Entity::find()
        .all(&app_state.db)
        .await?
        .iter()
        .map(|permission| PermissionSerializer::from(permission.clone()))
        .collect();

    Ok(JsonResponse::data(permissions, None))
}

#[axum::debug_handler]
pub async fn create_permission(
    State(app_state): State<Arc<AppState>>,
    Extension(user_model): Extension<user::Model>,
    Json(payload): Json<CreatePermissionRequest>,
) -> Result<impl IntoResponse, AppError> {
    AuthService::has_permission(&app_state, &user_model, "create_permission").await?;

    payload.validate()?;

    let permission: PermissionSerializer = payload
        .into_active_model()
        .insert(&app_state.db)
        .await?
        .into();

    Ok(JsonResponse::data(permission, None))
}

pub async fn get_permission(
    State(app_state): State<Arc<AppState>>,
    Path(permission_id): Path<i32>,
    Extension(user_model): Extension<user::Model>,
) -> Result<impl IntoResponse, AppError> {
    AuthService::has_permission(&app_state, &user_model, "read_permission").await?;

    let permission_serializer: PermissionSerializer = permission::Entity::find_by_id(permission_id)
        .one(&app_state.db)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?
        .into();

    Ok(JsonResponse::data(permission_serializer, None))
}
pub async fn update_permission(
    State(app_state): State<Arc<AppState>>,
    Path(permission_id): Path<i32>,
    Extension(user_model): Extension<user::Model>,
    Json(payload): Json<CreatePermissionRequest>,
) -> Result<impl IntoResponse, AppError> {
    AuthService::has_permission(&app_state, &user_model, "update_permission").await?;

    let permission = permission::Entity::find_by_id(permission_id)
        .one(&app_state.db)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;

    payload.validate()?;

    let mut permission: permission::ActiveModel = permission.into();

    permission.name = Set(payload.name);
    permission.code_name = Set(payload.code_name);

    let permission_serializer: PermissionSerializer =
        permission.update(&app_state.db).await?.into();

    Ok(JsonResponse::data(permission_serializer, None))
}
pub async fn delete_permission(
    State(app_state): State<Arc<AppState>>,
    Path(permission_id): Path<i32>,
    Extension(user_model): Extension<user::Model>,
) -> Result<impl IntoResponse, AppError> {
    AuthService::has_permission(&app_state, &user_model, "delete_permission").await?;

    let res = permission::Entity::delete_by_id(permission_id)
        .exec(&app_state.db)
        .await?;

    println!("{:?}", res);

    Ok(JsonResponse::data(
        None::<String>,
        Some("Permission deleted successfully".to_string()),
    ))
}
