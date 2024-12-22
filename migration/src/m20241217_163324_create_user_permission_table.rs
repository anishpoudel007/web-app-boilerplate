use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UserPermission::Table)
                    .if_not_exists()
                    .col(pk_auto(UserPermission::Id))
                    .col(integer(UserPermission::UserId))
                    .col(integer(UserPermission::PermissionId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-user-permission-user_id")
                            .from(UserPermission::Table, UserPermission::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-user-permission-permission_id")
                            .from(UserPermission::Table, UserPermission::PermissionId)
                            .to(Permission::Table, Permission::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserPermission::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum UserPermission {
    Table,
    Id,
    UserId,
    PermissionId,
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Permission {
    Table,
    Id,
}
