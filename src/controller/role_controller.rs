use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::get,
    Extension, Router,
};
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait, IntoActiveModel, Set};
use validator::Validate;

use crate::{
    api_response::JsonResponse,
    auth::auth_service::AuthService,
    error::AppError,
    extractor::ValidJson,
    form::role_form::{CreateRoleRequest, UpdateRoleRequest},
    models::_entities::{role, user},
    repository::{Repository, RoleRepository},
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
    Extension(user_model): Extension<user::Model>,
) -> Result<impl IntoResponse, AppError> {
    AuthService::has_permission(&app_state, &user_model, "read_roles").await?;

    let roles: Vec<RoleSerializer> = role::Entity::find()
        .all(&app_state.db)
        .await?
        .into_iter()
        .map(RoleSerializer::from)
        .collect();

    Ok(JsonResponse::data(roles, None))
}

#[axum::debug_handler]
pub async fn create_role(
    State(app_state): State<Arc<AppState>>,
    Extension(user_model): Extension<user::Model>,
    ValidJson(payload): ValidJson<CreateRoleRequest>,
) -> Result<impl IntoResponse, AppError> {
    AuthService::has_permission(&app_state, &user_model, "create_role").await?;

    payload.validate()?;

    let role_repo = RoleRepository;

    let role_active_model = payload.clone().into_active_model();

    role_repo
        .create(&app_state.db, role_active_model)
        .await
        .unwrap();

    // let role: RoleSerializer = payload
    //     .into_active_model()
    //     .insert(&app_state.db)
    //     .await?
    //     .into();

    Ok(())
    // Ok(JsonResponse::data(role, None))
}

pub async fn get_role(
    State(app_state): State<Arc<AppState>>,
    Path(role_id): Path<i32>,
    Extension(user_model): Extension<user::Model>,
) -> Result<impl IntoResponse, AppError> {
    AuthService::has_permission(&app_state, &user_model, "read_role").await?;

    let role_serializer: RoleSerializer = role::Entity::find_by_id(role_id)
        .one(&app_state.db)
        .await?
        .ok_or(DbErr::RecordNotFound("Role not found.".to_string()))?
        .into();

    Ok(JsonResponse::data(role_serializer, None))
}
pub async fn update_role(
    State(app_state): State<Arc<AppState>>,
    Path(role_id): Path<i32>,
    Extension(user_model): Extension<user::Model>,
    ValidJson(payload): ValidJson<UpdateRoleRequest>,
) -> Result<impl IntoResponse, AppError> {
    AuthService::has_permission(&app_state, &user_model, "update_role").await?;

    let role = role::Entity::find_by_id(role_id)
        .one(&app_state.db)
        .await?
        .ok_or(DbErr::RecordNotFound("Role not found.".to_string()))?;

    payload.validate()?;

    let mut role: role::ActiveModel = role.into();

    role.name = Set(payload.name);

    let role_serializer: RoleSerializer = role.update(&app_state.db).await?.into();

    Ok(JsonResponse::data(role_serializer, None))
}
pub async fn delete_role(
    State(app_state): State<Arc<AppState>>,
    Path(role_id): Path<i32>,
    Extension(user_model): Extension<user::Model>,
) -> Result<impl IntoResponse, AppError> {
    AuthService::has_permission(&app_state, &user_model, "delete_role").await?;

    let res = role::Entity::delete_by_id(role_id)
        .exec(&app_state.db)
        .await?;

    tracing::info!("{:?}", res);

    Ok(JsonResponse::data(
        None::<String>,
        Some("Role deleted successfully".to_string()),
    ))
}
