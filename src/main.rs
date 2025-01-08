use std::sync::Arc;

use axum::{http::StatusCode, Router};
use controller::{
    auth_controller, permission_controller, role_controller, user_controller, user_role_controller,
};
use sea_orm::{Database, DatabaseConnection};
use serde::Deserialize;
use tokio::{net::TcpListener, signal};
use tower_http::trace::TraceLayer;

mod api_response;
mod auth;
mod controller;
mod error;
mod extractor;
mod form;
mod mails;
mod middlewares;
mod models;
mod serializer;
mod utils;

#[derive(Debug, Deserialize, Clone)]
struct AppConfig {
    server_address: String,
    database_url: String,
    per_page: i32,
    jwt_secret: String,
    access_token_expiration_minutes: i64,
    refresh_token_expiration_minutes: i64,
    smtp_host: String,
    smtp_username: String,
    smtp_password: String,
    from_email: String,
}

#[derive(Clone, Debug)]
struct AppState {
    db: DatabaseConnection,
    config: AppConfig,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect("Unable to access .env file");

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .pretty()
        .with_ansi(true)
        .init();

    let app_config: AppConfig = config::Config::builder()
        .add_source(config::Environment::default())
        .build()
        .unwrap()
        .try_deserialize()
        .unwrap();

    tracing::info!("{:#?}", app_config);

    tracing::info!("Listening on {}", app_config.server_address);

    let listener = TcpListener::bind(&app_config.server_address)
        .await
        .expect("Could not create TCP Listener");

    let app = create_app(app_config).await;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn create_app(app_config: AppConfig) -> Router {
    let db = Database::connect(&app_config.database_url)
        .await
        .expect("Cannot connect to a database");

    let app_state = Arc::new(AppState {
        db,
        config: app_config,
    });

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
}

async fn fallback_handler() -> StatusCode {
    StatusCode::NOT_FOUND
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use dotenvy::dotenv;
    use tower::{Service, ServiceExt};

    #[tokio::test]
    async fn hello_world() {
        dotenv().ok();

        let builder = config::Config::builder()
            .add_source(config::Environment::default())
            .build()
            .unwrap();

        let app_config: AppConfig = builder.try_deserialize().unwrap();

        let app = create_app(app_config).await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/tasks")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
