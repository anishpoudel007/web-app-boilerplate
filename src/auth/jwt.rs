use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub iat: usize,
    pub exp: usize,
}

#[derive(Debug, Serialize)]
pub struct UserToken {
    pub access_token: String,
    pub refresh_token: Option<String>,
}

pub async fn create_user_token(subject: &str, expire_in_minutes: i64) -> String {
    let now = Utc::now();
    let iat = now.timestamp() as usize;
    let exp = (now + Duration::minutes(expire_in_minutes)).timestamp() as usize;

    let token_claims = TokenClaims {
        sub: subject.to_string(),
        iat,
        exp,
    };

    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT Secret not set.");

    let access_token = encode(
        &Header::default(),
        &token_claims,
        &EncodingKey::from_secret(jwt_secret.as_ref()),
    )
    .expect("Cannot encode user token");

    access_token
}
