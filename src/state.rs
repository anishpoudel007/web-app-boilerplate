use crate::configgg::AppConfig;
use sea_orm::DatabaseConnection;

#[derive(Clone, Debug)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub config: AppConfig,
}
