use std::sync::Arc;

use crate::{
    api_response::JsonResponse,
    auth::jwt::{create_user_token, UserToken},
    error::AppError,
    extractor::ValidJson,
    form::user_form::{CreateUserRequest, UserLogin},
    mails::auth_mails::send_register_mail,
    models::_entities::{user, user_profile},
    serializer::UserWithProfileSerializer,
    utils::verify_password,
    AppState,
};

use axum::{extract::State, response::IntoResponse, routing::post, Json, Router};
use sea_orm::{
    ActiveModelTrait as _, ColumnTrait, Condition, EntityTrait, QueryFilter, Set,
    TransactionTrait as _,
};
use validator::Validate as _;

pub async fn get_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/auth/login", post(login))
        .route("/auth/logout", post(logout))
}

pub async fn get_login_route() -> Router<Arc<AppState>> {
    Router::new()
        .route("/login", post(login))
        .route("/register", post(register))
}

pub async fn get_logout_route() -> Router<Arc<AppState>> {
    Router::new().route("/logout", post(logout))
}

#[axum::debug_handler]
pub async fn register(
    State(app_state): State<Arc<AppState>>,
    ValidJson(payload): ValidJson<CreateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
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

    let user_email = payload.email.clone();

    let user_with_profile = app_state
        .db
        .transaction::<_, (user::Model, Option<user_profile::Model>), sea_orm::DbErr>(|txn| {
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
        .map_err(|e| {
            println!("{:#?}", e);
            AppError::GenericError(e.to_string())
        })?; // should be database error

    let user_serializer = UserWithProfileSerializer::from(user_with_profile);

    println!("{:#?}", user_serializer);

    send_register_mail(app_state, "User Registration Complete", &user_email)
        .map_err(AppError::GenericError)?;

    Ok(JsonResponse::data(user_serializer, None))
}

#[axum::debug_handler]
pub async fn login(
    State(app_state): State<Arc<AppState>>,
    ValidJson(payload): ValidJson<UserLogin>,
) -> Result<impl IntoResponse, AppError> {
    payload.validate()?;

    let user = user::Entity::find()
        .filter(user::Column::Username.eq(payload.username))
        .one(&app_state.db)
        .await?
        .ok_or(AppError::GenericError("User not found.".to_string()))?;

    if !verify_password(&user.password, &payload.password)? {
        return Err(AppError::GenericError("Invalid user".to_string()));
    }

    let app_config = app_state.config.to_owned();

    let access_token = create_user_token(
        &user.email,
        app_config.access_token_expiration_minutes,
        &app_config.jwt_secret,
    )
    .await
    .map_err(AppError::GenericError)?;

    let refresh_token = create_user_token(
        &user.email,
        app_config.refresh_token_expiration_minutes,
        &app_config.jwt_secret,
    )
    .await
    .map_err(AppError::GenericError)?;

    let user_token = UserToken {
        access_token,
        refresh_token: Some(refresh_token),
    };

    Ok(JsonResponse::data(user_token, None))
}

pub async fn logout() {}
