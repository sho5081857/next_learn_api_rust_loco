use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Invoices::Table)
                    .col(uuid_uniq(Invoices::Id).primary_key())
                    .col(uuid(Invoices::CustomerId))
                    .col(integer(Invoices::Amount))
                    .col(string(Invoices::Status))
                    .col(date(Invoices::Date))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Invoices::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Invoices {
    Table,
    Id,
    CustomerId,
    Amount,
    Status,
    Date,

}
