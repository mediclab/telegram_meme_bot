use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Users::UserId).big_integer().not_null().primary_key())
                    .col(ColumnDef::new(Users::Username).string_len(33).null())
                    .col(ColumnDef::new(Users::Firstname).string_len(65).not_null())
                    .col(ColumnDef::new(Users::Lastname).string_len(65).null())
                    .col(ColumnDef::new(Users::DeletedAt).timestamp().null())
                    .col(
                        ColumnDef::new(Users::CreatedAt)
                            .timestamp()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(Users::Table).to_owned()).await
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    UserId,
    Username,
    Firstname,
    Lastname,
    DeletedAt,
    CreatedAt,
}
