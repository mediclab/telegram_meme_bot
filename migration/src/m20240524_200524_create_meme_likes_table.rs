use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table_exists = manager.has_table(MemeLikes::Table.to_string()).await?;

        manager
            .create_table(
                Table::create()
                    .table(MemeLikes::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(MemeLikes::Uuid)
                            .uuid()
                            .not_null()
                            .default(Expr::cust("gen_random_uuid()"))
                            .primary_key(),
                    )
                    .col(ColumnDef::new(MemeLikes::MemeUuid).uuid().not_null())
                    .col(ColumnDef::new(MemeLikes::UserId).big_integer().not_null())
                    .col(ColumnDef::new(MemeLikes::Num).small_integer().not_null().default(1))
                    .col(
                        ColumnDef::new(MemeLikes::CreatedAt)
                            .timestamp()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;
        if !table_exists {
            manager
                .create_foreign_key(
                    ForeignKey::create()
                        .name("meme_likes_meme_uuid_fkey")
                        .from(MemeLikes::Table, MemeLikes::MemeUuid)
                        .to(Memes::Table, Memes::Uuid)
                        .on_delete(ForeignKeyAction::Cascade)
                        .to_owned(),
                )
                .await?;
        }
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("meme_likes_user_id_meme_uuid_idx")
                    .table(MemeLikes::Table)
                    .col(MemeLikes::UserId)
                    .col(MemeLikes::MemeUuid)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .table(Memes::Table)
                    .if_exists()
                    .name("meme_likes_user_id_meme_uuid_idx")
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(MemeLikes::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum MemeLikes {
    Table,
    Uuid,
    UserId,
    MemeUuid,
    Num,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Memes {
    Table,
    Uuid,
}
