use sea_orm_migration::prelude::extension::postgres::Type;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(Alias::new("MessageType"))
                    .values([
                        MessageTypes::Text,
                        MessageTypes::Photo,
                        MessageTypes::Video,
                        MessageTypes::Document,
                    ])
                    .to_owned(),
            )
            .await?;
        manager
            .create_type(
                Type::create()
                    .as_enum(Alias::new("MessageEntityType"))
                    .values([
                        EntityTypes::MemeAlreadyExists,
                        EntityTypes::SimilarMeme,
                        EntityTypes::NewbieUser,
                        EntityTypes::UserLeftChat,
                        EntityTypes::PressFToPrayRespects,
                    ])
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Messages::Table)
                    .add_column(
                        ColumnDef::new(Messages::Type)
                            .custom(Alias::new("\"MessageType\""))
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Messages::Table)
                    .add_column(
                        ColumnDef::new(Messages::EntityType)
                            .custom(Alias::new("\"MessageEntityType\""))
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Messages::Table)
                    .modify_column(ColumnDef::new(Messages::Message).not_null())
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Messages::Table)
                    .drop_column(Messages::MessageType)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Messages::Table)
                    .drop_column(Messages::EntityType)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Messages::Table)
                    .drop_column(Messages::Type)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_type(Type::drop().name(Alias::new("MessageType")).to_owned())
            .await?;
        manager
            .drop_type(Type::drop().name(Alias::new("MessageEntityType")).to_owned())
            .await?;
        manager
            .truncate_table(Table::truncate().table(Messages::Table).to_owned())
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Messages::Table)
                    .add_column(ColumnDef::new(Messages::MessageType).string_len(255).not_null())
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum MessageTypes {
    Text,
    Photo,
    Video,
    Document,
}

#[derive(DeriveIden)]
enum EntityTypes {
    MemeAlreadyExists,
    SimilarMeme,
    NewbieUser,
    UserLeftChat,
    PressFToPrayRespects,
}

#[derive(DeriveIden)]
enum Messages {
    Table,
    Type,
    Message,
    MessageType,
    EntityType,
}
