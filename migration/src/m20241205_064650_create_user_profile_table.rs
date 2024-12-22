use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UserProfile::Table)
                    .if_not_exists()
                    .col(pk_auto(UserProfile::Id))
                    .col(integer(UserProfile::UserId))
                    .col(string_null(UserProfile::Address))
                    .col(string_null(UserProfile::MobileNumber))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-user-profile-user_id")
                            .from(UserProfile::Table, UserProfile::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserProfile::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum UserProfile {
    Table,
    Id,
    #[sea_orm(iden = "user_id")]
    UserId,
    Address,
    #[sea_orm(iden = "mobile_number")]
    MobileNumber,
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
}
