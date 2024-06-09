use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ChatAdmins::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ChatAdmins::Uuid)
                            .uuid()
                            .not_null()
                            .default(Expr::cust("gen_random_uuid()"))
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ChatAdmins::ChatId).big_integer().not_null())
                    .col(ColumnDef::new(ChatAdmins::UserId).big_integer().not_null())
                    .col(
                        ColumnDef::new(ChatAdmins::CreatedAt)
                            .timestamp()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("chat_admins_chat_id_user_id_idx")
                    .table(ChatAdmins::Table)
                    .col(ChatAdmins::ChatId)
                    .col(ChatAdmins::UserId)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .table(ChatAdmins::Table)
                    .if_exists()
                    .name("chat_admins_chat_id_user_id_idx")
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(ChatAdmins::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ChatAdmins {
    Table,
    Uuid,
    ChatId,
    UserId,
    CreatedAt,
}
