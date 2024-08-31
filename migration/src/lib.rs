#![allow(elided_lifetimes_in_paths)]
#![allow(clippy::wildcard_imports)]
pub use sea_orm_migration::prelude::*;

mod m20220101_000001_users;
mod m20231103_114510_notes;

mod m20240630_143949_revenues;
mod m20240701_122134_customers;
mod m20240707_160715_invoices;
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_users::Migration),
            Box::new(m20231103_114510_notes::Migration),
            Box::new(m20240630_143949_revenues::Migration),
            Box::new(m20240701_122134_customers::Migration),
            Box::new(m20240707_160715_invoices::Migration),
        ]
    }
}