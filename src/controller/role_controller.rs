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
    api_response::JsonResponse,
    error::AppError,
    form::role_form::{CreateRoleRequest, UpdateRoleRequest},
    models::_entities::role,
    serializer::RoleSerializer,
    AppState,
};

pub async fn get_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(get_roles).post(create_role))
        .route(
            "/{role_id}",
            get(get_role).put(update_role).delete(delete_role),
        )
}

#[axum::debug_handler]
pub async fn get_roles(
    State(app_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let roles: Vec<RoleSerializer> = role::Entity::find()
        .all(&app_state.db)
        .await?
        .iter()
        .map(|role| RoleSerializer::from(role.clone()))
        .collect();

    Ok(JsonResponse::data(roles, None))
}

#[axum::debug_handler]
pub async fn create_role(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<CreateRoleRequest>,
) -> Result<impl IntoResponse, AppError> {
    payload.validate()?;

    let role: RoleSerializer = payload
        .into_active_model()
        .insert(&app_state.db)
        .await?
        .into();

    Ok(JsonResponse::data(role, None))
}

pub async fn get_role(
    State(app_state): State<Arc<AppState>>,
    Path(role_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    let role_serializer: RoleSerializer = role::Entity::find_by_id(role_id)
        .one(&app_state.db)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?
        .into();

    Ok(JsonResponse::data(role_serializer, None))
}
pub async fn update_role(
    State(app_state): State<Arc<AppState>>,
    Path(role_id): Path<i32>,
    Json(payload): Json<UpdateRoleRequest>,
) -> Result<impl IntoResponse, AppError> {
    let role = role::Entity::find_by_id(role_id)
        .one(&app_state.db)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;

    payload.validate()?;

    let mut role: role::ActiveModel = role.into();

    role.name = Set(payload.name);

    let role_serializer: RoleSerializer = role.update(&app_state.db).await?.into();

    Ok(JsonResponse::data(role_serializer, None))
}
pub async fn delete_role(
    State(app_state): State<Arc<AppState>>,
    Path(role_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    let res = role::Entity::delete_by_id(role_id)
        .exec(&app_state.db)
        .await?;

    println!("{:?}", res);

    Ok(JsonResponse::data(
        None::<String>,
        Some("Role deleted successfully".to_string()),
    ))
}
