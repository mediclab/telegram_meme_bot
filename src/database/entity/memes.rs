//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.15

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "memes")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub uuid: Uuid,
    pub msg_id: Option<i64>,
    pub user_id: i64,
    pub chat_id: i64,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub photos: Option<Json>,
    pub posted_at: Option<DateTime>,
    pub updated_at: Option<DateTime>,
    pub long_hash: Option<String>,
    pub short_hash: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::meme_likes::Entity")]
    MemeLikes,
}

impl Related<super::meme_likes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MemeLikes.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
