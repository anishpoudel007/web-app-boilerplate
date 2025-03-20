use std::sync::Arc;

use crate::controller::{
    auth_controller, permission_controller, role_controller, user_controller, user_role_controller,
};
use crate::{middlewares, state::AppState};
use axum::http::{HeaderValue, Method, StatusCode};
use axum::middleware;
use axum::{routing::get, Router};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

pub async fn create_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .nest("/api/users", user_controller::get_routes().await)
        .nest(
            "/api/permissions",
            permission_controller::get_routes().await,
        )
        .nest("/api/roles", role_controller::get_routes().await)
        .nest("/api/user_roles", user_role_controller::get_routes().await)
        // .nest("/api", controller::auth_controller::get_routes().await)
        .nest("/api/auth", auth_controller::get_logout_route().await)
        .route_layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            middlewares::auth_guard::auth_guard,
        ))
        .nest("/api/auth", auth_controller::get_login_route().await)
        .with_state(app_state)
        .fallback(fallback_handler)
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin("http://localhost:8080".parse::<HeaderValue>().unwrap())
                .allow_methods([Method::GET]),
        )
}

async fn fallback_handler() -> StatusCode {
    StatusCode::NOT_FOUND
}
