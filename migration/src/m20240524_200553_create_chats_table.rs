use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Chats::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Chats::ChatId).big_integer().not_null().primary_key())
                    .col(ColumnDef::new(Chats::Chatname).string_len(256).null())
                    .col(ColumnDef::new(Chats::Title).string_len(256).null())
                    .col(ColumnDef::new(Chats::Description).string_len(256).null())
                    .col(ColumnDef::new(Chats::DeletedAt).timestamp().null())
                    .col(
                        ColumnDef::new(Chats::CreatedAt)
                            .timestamp()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(Chats::Table).to_owned()).await
    }
}

#[derive(DeriveIden)]
enum Chats {
    Table,
    ChatId,
    Chatname,
    Title,
    Description,
    DeletedAt,
    CreatedAt,
}
