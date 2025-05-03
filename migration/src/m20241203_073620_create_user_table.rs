use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(pk_auto(User::Id))
                    .col(string(User::Name))
                    .col(string_uniq(User::Username).string_len(100))
                    .col(string_uniq(User::Email))
                    .col(string(User::Password))
                    .col(boolean(User::IsSuperadmin).default(false))
                    .col(date_time(User::DateCreated))
                    .col(date_time_null(User::DateUpdated))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
    Name,
    Username,
    Email,
    Password,
    IsSuperadmin,
    DateCreated,
    DateUpdated,
}
