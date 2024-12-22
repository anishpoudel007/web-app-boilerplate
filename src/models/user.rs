use sea_orm::{ActiveModelBehavior, ConnectionTrait, DbErr, Related, RelationDef, RelationTrait};

use super::_entities::{
    permission, role,
    user::{ActiveModel, Entity},
    user_permission, user_role,
};

impl Related<role::Entity> for Entity {
    fn to() -> RelationDef {
        user_role::Relation::Role.def()
    }
    fn via() -> Option<RelationDef> {
        Some(user_role::Relation::User.def().rev())
    }
}

impl Related<permission::Entity> for Entity {
    fn to() -> RelationDef {
        user_permission::Relation::Permission.def()
    }
    fn via() -> Option<RelationDef> {
        Some(user_permission::Relation::User.def().rev())
    }
}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(self, db: &C, insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        let now = chrono::Utc::now().naive_utc();

        if insert && self.date_created.is_not_set() {
            let mut this = self;
            this.date_created = sea_orm::ActiveValue::Set(now);
            Ok(this)
        } else if !insert && self.date_updated.is_unchanged() {
            let mut this = self;
            this.date_updated = sea_orm::ActiveValue::Set(Some(now));
            Ok(this)
        } else {
            Ok(self)
        }
    }
}
