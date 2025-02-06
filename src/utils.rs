use std::sync::Arc;

use hmac::{self, Hmac, Mac};
use jsonwebtoken::{decode, DecodingKey, Validation};
use sea_orm::{ColumnTrait, DbErr};
use sea_orm::{EntityTrait, QueryFilter};
use sha2::Sha256;

use crate::AppState;
use crate::{auth::jwt::TokenClaims, error::AppError, models::_entities::user};

pub fn hash(text: &str) -> String {
    let mut mac: Hmac<Sha256> =
        Hmac::new_from_slice(b"secret_key").expect("HMAC can take key of any size");

    mac.update(text.as_bytes());

    let result = mac.finalize();
    let code_bytes = result.into_bytes();

    hex::encode(code_bytes)
}

pub fn verify_password(hex_code: &str, to_verify: &str) -> Result<bool, AppError> {
    let mut mac: Hmac<Sha256> =
        Hmac::new_from_slice(b"secret_key").expect("HMAC can take key of any size");

    mac.update(to_verify.as_bytes());

    let result = mac.finalize();

    let code_byte = hex::decode(hex_code).map_err(|err| AppError::GenericError(err.to_string()))?;

    Ok(code_byte[..] == result.into_bytes()[..])
}

pub async fn verify_token(app_state: Arc<AppState>, token: &str) -> Result<user::Model, AppError> {
    let token_claim = decode::<TokenClaims>(
        token,
        &DecodingKey::from_secret(app_state.config.jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|_| AppError::GenericError("Invalid Token".to_string()))?;

    let user = user::Entity::find()
        .filter(user::Column::Email.eq(token_claim.claims.sub))
        .one(&app_state.db)
        .await?
        .ok_or(DbErr::RecordNotFound("User not found.".to_string()))?;

    Ok(user)
}
