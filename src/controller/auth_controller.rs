use std::sync::Arc;

use crate::{
    api_response::JsonResponse,
    auth::jwt::{create_user_token, UserToken},
    error::AppError,
    form::user_form::UserLogin,
    models::_entities::user,
    utils::verify_password,
    AppState,
};

use axum::{extract::State, response::IntoResponse, routing::post, Json, Router};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

pub async fn get_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/auth/login", post(login))
        .route("/auth/logout", post(logout))
}

pub async fn get_login_route() -> Router<Arc<AppState>> {
    Router::new().route("/login", post(login))
}

pub async fn get_logout_route() -> Router<Arc<AppState>> {
    Router::new().route("/logout", post(logout))
}

#[axum::debug_handler]
pub async fn login(
    State(app_state): State<Arc<AppState>>,
    Json(user_login): Json<UserLogin>,
) -> Result<impl IntoResponse, AppError> {
    let user = user::Entity::find()
        .filter(user::Column::Username.eq(user_login.username))
        .one(&app_state.db)
        .await?
        .ok_or(AppError::GenericError("User not found.".to_string()))?;

    if !verify_password(&user.password, &user_login.password)? {
        return Err(AppError::GenericError("Invalid user".to_string()));
    }

    let access_token = create_user_token(&user.email, 10).await;
    let refresh_token = create_user_token(&user.email, 1440).await;

    let user_token = UserToken {
        access_token,
        refresh_token: Some(refresh_token),
    };

    Ok(JsonResponse::data(user_token, None))
}

pub async fn logout() {}
