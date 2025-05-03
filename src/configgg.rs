use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub app_debug: bool,
    pub server_address: String,
    pub database_url: String,
    pub per_page: i32,
    pub jwt_secret: String,
    pub access_token_expiration_minutes: i64,
    pub refresh_token_expiration_minutes: i64,
    pub smtp_host: String,
    pub smtp_username: String,
    pub smtp_password: String,
    pub from_email: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        config::Config::builder()
            .add_source(config::Environment::default())
            .build()?
            .try_deserialize()
    }
}
