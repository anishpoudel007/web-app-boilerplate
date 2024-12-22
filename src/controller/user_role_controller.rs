use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, routing::get, Router};
use sea_orm::EntityTrait;

use crate::{
    api_response::JsonResponse,
    error::AppError,
    models::_entities::{role, user},
    AppState,
};

pub async fn get_routes() -> Router<Arc<AppState>> {
    Router::new().route("/", get(get_user_roles))
}

pub async fn get_user_roles(
    State(app_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let user_with_roles = user::Entity::find()
        .find_with_related(role::Entity)
        .all(&app_state.db)
        .await?;

    Ok(JsonResponse::data(user_with_roles, None))
}
