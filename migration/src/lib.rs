pub use sea_orm_migration::prelude::*;

mod m20241203_073620_create_user_table;
mod m20241205_064650_create_user_profile_table;
mod m20241216_055637_create_permission_table;
mod m20241216_092524_create_role_table;
mod m20241216_095114_create_user_role_table;
mod m20241217_163324_create_user_permission_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20241203_073620_create_user_table::Migration),
            Box::new(m20241205_064650_create_user_profile_table::Migration),
            Box::new(m20241216_055637_create_permission_table::Migration),
            Box::new(m20241216_092524_create_role_table::Migration),
            Box::new(m20241216_095114_create_user_role_table::Migration),
            Box::new(m20241217_163324_create_user_permission_table::Migration),
        ]
    }
}
