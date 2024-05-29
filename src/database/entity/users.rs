//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.15

use crate::database::Database;
use chrono::Utc;
use sea_orm::entity::prelude::*;
use sea_orm::Set;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub user_id: i64,
    pub username: Option<String>,
    pub firstname: String,
    pub lastname: Option<String>,
    pub deleted_at: Option<DateTime>,
    pub created_at: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Entity {
    pub async fn delete(user_id: i64) -> bool {
        Entity::update(ActiveModel {
            user_id: Set(user_id),
            deleted_at: Set(Some(Utc::now().naive_utc())),
            ..Default::default()
        })
        .exec(Database::global().connection())
        .await
        .is_ok()
    }
}