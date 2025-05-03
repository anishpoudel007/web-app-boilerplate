use std::sync::Arc;

use configgg::AppConfig;
use routes::create_router;
use state::AppState;
use tokio::{net::TcpListener, signal};
use utils::connect_to_database;

mod api_response;
mod auth;
mod configgg;
mod controller;
mod error;
mod extractor;
mod form;
mod mails;
mod middlewares;
mod models;
mod repository;
mod routes;
mod serializer;
mod service;
mod state;
mod utils;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect("Unable to access .env file");

    let app_config: AppConfig = AppConfig::from_env().expect("Failed to load configuration");

    if app_config.app_debug {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .pretty()
            .with_ansi(true)
            .init();
    }

    tracing::info!("{:#?}", app_config);

    // Initialize database connection
    let db = connect_to_database(&app_config.database_url)
        .await
        .expect("Failed to connect to database");

    // Create application state
    let app_state = Arc::new(AppState {
        db,
        config: app_config.clone(),
    });

    // Create the Axum router
    let app = create_router(app_state).await;

    // Start the server
    let listener = TcpListener::bind(&app_config.server_address)
        .await
        .expect("Could not create TCP Listener");

    tracing::info!("Listening on {}", app_config.server_address);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
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
mod tests {}
