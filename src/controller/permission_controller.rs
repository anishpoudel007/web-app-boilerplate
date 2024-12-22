use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel, Set};
use validator::Validate;

use crate::{
    api_response::JsonResponse, error::AppError, form::permission_form::CreatePermissionRequest,
    models::_entities::permission, serializer::PermissionSerializer, AppState,
};

pub async fn get_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(get_permissions).post(create_permission))
        .route(
            "/:permission_id",
            get(get_permission)
                .put(update_permission)
                .delete(delete_permission),
        )
}

#[axum::debug_handler]
pub async fn get_permissions(
    State(app_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
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
    Json(permission_request): Json<CreatePermissionRequest>,
) -> Result<impl IntoResponse, AppError> {
    permission_request.validate()?;

    let permission: PermissionSerializer = permission_request
        .into_active_model()
        .insert(&app_state.db)
        .await?
        .into();

    Ok(JsonResponse::data(permission, None))
}

pub async fn get_permission(
    State(app_state): State<Arc<AppState>>,
    Path(permission_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
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
    Json(permission_request): Json<CreatePermissionRequest>,
) -> Result<impl IntoResponse, AppError> {
    let permission = permission::Entity::find_by_id(permission_id)
        .one(&app_state.db)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;

    permission_request.validate()?;

    let mut permission: permission::ActiveModel = permission.into();

    permission.name = Set(permission_request.name);
    permission.code_name = Set(permission_request.code_name);

    let permission_serializer: PermissionSerializer =
        permission.update(&app_state.db).await?.into();

    Ok(JsonResponse::data(permission_serializer, None))
}
pub async fn delete_permission(
    State(app_state): State<Arc<AppState>>,
    Path(permission_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    let res = permission::Entity::delete_by_id(permission_id)
        .exec(&app_state.db)
        .await?;

    println!("{:?}", res);

    Ok(JsonResponse::data(
        None::<String>,
        Some("Permission deleted successfully".to_string()),
    ))
}
