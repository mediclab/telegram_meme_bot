//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.15

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "meme_likes")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub uuid: Uuid,
    pub meme_uuid: Option<Uuid>,
    pub user_id: i64,
    pub num: i16,
    pub created_at: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::memes::Entity",
        from = "Column::MemeUuid",
        to = "super::memes::Column::Uuid",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Memes,
}

impl Related<super::memes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Memes.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
