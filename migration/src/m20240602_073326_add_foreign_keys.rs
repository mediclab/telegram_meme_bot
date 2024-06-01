use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("meme_likes_user_id_fkey")
                    .from(MemeLikes::Table, MemeLikes::UserId)
                    .to(Users::Table, Users::UserId)
                    .on_delete(ForeignKeyAction::NoAction)
                    .to_owned(),
            )
            .await?;
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("memes_user_id_fkey")
                    .from(Memes::Table, Memes::UserId)
                    .to(Users::Table, Users::UserId)
                    .on_delete(ForeignKeyAction::NoAction)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(ForeignKey::drop().name("memes_user_id_fkey").to_owned())
            .await?;
        manager
            .drop_foreign_key(ForeignKey::drop().name("meme_likes_user_id_fkey").to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum MemeLikes {
    Table,
    UserId,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    UserId,
}

#[derive(DeriveIden)]
enum Memes {
    Table,
    UserId,
}
