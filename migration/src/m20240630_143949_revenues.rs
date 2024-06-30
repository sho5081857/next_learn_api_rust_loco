use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Revenues::Table)
                    .if_not_exists()
                    .col(string(Revenues::Month))
                    .col(integer(Revenues::Revenue))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Revenues::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Revenues {
    Table,
    Month,
    Revenue,
}
