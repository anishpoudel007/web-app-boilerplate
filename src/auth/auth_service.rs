use std::sync::Arc;

use sea_orm::{ColumnTrait as _, ModelTrait as _, PaginatorTrait as _, QueryFilter as _};

use crate::{
    error::AppError,
    models::_entities::{permission, role, user},
    AppState,
};

pub struct AuthService;

impl AuthService {
    pub async fn has_role(
        ctx: &Arc<AppState>,
        user: &user::Model,
        role: &str,
    ) -> Result<(), AppError> {
        if user.is_superadmin {
            return Ok(());
        }

        let count = user
            .find_related(role::Entity)
            .filter(role::Column::Name.contains(role))
            .count(&ctx.db)
            .await?;

        if count == 0 {
            return Err(AppError::Unauthorized);
        }

        Ok(())
    }

    pub async fn has_permission(
        ctx: &Arc<AppState>,
        user: &user::Model,
        permission: &str,
    ) -> Result<(), AppError> {
        if user.is_superadmin {
            return Ok(());
        }

        let count = user
            .find_related(permission::Entity)
            .filter(permission::Column::Name.contains(permission))
            .count(&ctx.db)
            .await?;

        if count == 0 {
            return Err(AppError::Unauthorized);
        }

        Ok(())
    }
}
