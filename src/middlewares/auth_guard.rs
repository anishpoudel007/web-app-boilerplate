use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::Response,
};

use crate::{error::AppError, utils::verify_token, AppState};

pub async fn auth_guard(
    State(app_state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| header.strip_prefix("Bearer "))
        .ok_or(AppError::GenericError(
            "No token found in header.".to_string(),
        ))?;

    let user = verify_token(app_state, token).await?;

    request.extensions_mut().insert(user);

    let response = next.run(request).await;

    Ok(response)
}
