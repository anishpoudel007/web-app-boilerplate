use std::sync::Arc;

use crate::{
    AppState,
    api_response::JsonResponse,
    auth::jwt::{UserToken, create_user_token},
    error::AppError,
    extractor::ValidJson,
    form::user_form::{CreateUserRequest, UserLogin},
    mails::auth_mails::send_register_mail,
    models::_entities::{user, user_profile},
    serializer::UserWithProfileSerializer,
    utils::verify_password,
};

use axum::{Router, extract::State, response::IntoResponse, routing::post};
use garde::Validate;
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
    payload.validate_with(&app_state)?;

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
            tracing::error!("{:#?}", e);
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{
        body::{Body, to_bytes},
        http::{self, Request, StatusCode, header},
    };
    use dotenvy::dotenv;
    use serde_json::json;
    use tower::ServiceExt as _;

    use crate::{
        api_response::ErrorResponse, configgg::AppConfig, routes::create_router, state::AppState,
        utils::connect_to_database,
    };

    #[tokio::test]
    async fn test_invalid_login() {
        dotenv().ok();

        let app_config = AppConfig::from_env().unwrap();

        let app = create_router(Arc::new(AppState {
            db: connect_to_database(&app_config.database_url).await.unwrap(),
            config: app_config,
        }))
        .await;

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/api/auth/login")
                    .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    // .body(Body::empty())
                    .body(Body::from(
                        json!({"username":"anish", "password":"password"}).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        let status = response.status();
        let body_bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();

        let error_response: ErrorResponse = serde_json::from_str(&body_str).unwrap();

        let expected_response = ErrorResponse {
            error: json!("Invalid user"),
            message: "Error".to_string(),
        };

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(error_response, expected_response);
    }
}
