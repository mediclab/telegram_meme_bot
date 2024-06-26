use super::meme_likes::{MemeLikeOperation, MemeLikesCountAll};
use crate::database::{entity::prelude::MemeLikes, Database};
use sea_orm::entity::prelude::*;
use sea_orm::{
    sea_query::{Alias, Order},
    JoinType, QueryOrder, QuerySelect, Set,
};

#[derive(DeriveIden)]
pub enum Memes {
    Table,
}

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
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::UserId",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Users,
}

impl Related<super::meme_likes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MemeLikes.def()
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Entity {
    pub async fn get_by_id(uuid: Uuid) -> Option<Model> {
        let res = Self::find_by_id(uuid).one(Database::global().connection()).await;

        res.unwrap_or_else(|e| {
            error!("Can't get meme from database: {e}");
            None
        })
    }

    pub async fn get_by_msg_id(chat_id: i64, msg_id: u64) -> Option<Model> {
        let res = Self::find()
            .filter(Column::ChatId.eq(chat_id))
            .filter(Column::MsgId.eq(msg_id))
            .one(Database::global().connection())
            .await;

        res.unwrap_or_else(|e| {
            error!("Can't get meme from database: {e}");
            None
        })
    }

    pub async fn get_by_short_hash(hash: &str) -> Vec<Model> {
        let res = Self::find()
            .filter(Column::ShortHash.eq(hash))
            .all(Database::global().connection())
            .await;

        res.unwrap_or_else(|e| {
            error!("Can't get memes by short hash from database: {e}");
            Vec::new()
        })
    }

    pub async fn get_count(chat_id: i64) -> u64 {
        let res = Self::find()
            .filter(Column::ChatId.eq(chat_id))
            .count(Database::global().connection())
            .await;

        res.unwrap_or_else(|e| {
            error!("Can't get meme count: {e}");
            0
        })
    }

    pub async fn get_max_liked(from: DateTimeUtc, to: DateTimeUtc) -> Option<Model> {
        let res = Self::find()
            .column_as(super::meme_likes::Column::Num.sum(), "likes")
            .join(JoinType::InnerJoin, Relation::MemeLikes.def())
            .filter(super::meme_likes::Column::CreatedAt.gt(from))
            .filter(super::meme_likes::Column::CreatedAt.lte(to))
            .filter(super::meme_likes::Column::Num.eq(MemeLikeOperation::Like.id()))
            .group_by(Column::Uuid)
            .group_by(Column::PostedAt)
            .having(Expr::expr(super::meme_likes::Column::Num.sum()).gt(0))
            .order_by(Expr::col(Alias::new("likes")), Order::Desc)
            .order_by(Column::PostedAt, Order::Desc)
            .one(Database::global().connection())
            .await;

        res.unwrap_or_else(|e| {
            error!("Can't get meme from database: {e}");
            None
        })
    }

    pub async fn get_max_disliked(from: DateTimeUtc, to: DateTimeUtc) -> Option<Model> {
        let res = Self::find()
            .column_as(super::meme_likes::Column::Num.sum(), "dislikes")
            .join(JoinType::InnerJoin, Relation::MemeLikes.def())
            .filter(super::meme_likes::Column::CreatedAt.gte(from))
            .filter(super::meme_likes::Column::CreatedAt.lte(to))
            .filter(super::meme_likes::Column::Num.eq(MemeLikeOperation::Dislike.id()))
            .group_by(Column::Uuid)
            .group_by(Column::PostedAt)
            .having(Expr::expr(super::meme_likes::Column::Num.sum()).lt(-4))
            .order_by(Expr::col(Alias::new("dislikes")), Order::Asc)
            .order_by(Column::PostedAt, Order::Desc)
            .one(Database::global().connection())
            .await;

        res.unwrap_or_else(|e| {
            error!("Can't get meme from database: {e}");
            None
        })
    }
}

impl Model {
    pub async fn replace_msg_id(&self, msg_id: i64) -> bool {
        let mut model: ActiveModel = self.clone().into();

        model.msg_id = Set(Some(msg_id));

        model.update(Database::global().connection()).await.is_ok()
    }

    pub async fn like(&self, from_user_id: i64) -> bool {
        MemeLikes::add(self.uuid, from_user_id, MemeLikeOperation::Like).await
    }

    pub async fn dislike(&self, from_user_id: i64) -> bool {
        MemeLikes::add(self.uuid, from_user_id, MemeLikeOperation::Dislike).await
    }

    pub async fn like_exists(&self, from_user_id: i64) -> bool {
        MemeLikes::exists(self.uuid, from_user_id, MemeLikeOperation::Like).await
    }

    pub async fn dislike_exists(&self, from_user_id: i64) -> bool {
        MemeLikes::exists(self.uuid, from_user_id, MemeLikeOperation::Dislike).await
    }

    pub async fn cancel_like(&self, from_user_id: i64) -> bool {
        MemeLikes::remove(self.uuid, from_user_id, MemeLikeOperation::Like).await
    }

    pub async fn cancel_dislike(&self, from_user_id: i64) -> bool {
        MemeLikes::remove(self.uuid, from_user_id, MemeLikeOperation::Dislike).await
    }

    pub async fn count_all_likes(&self) -> Option<MemeLikesCountAll> {
        MemeLikes::count_all(Some(self.uuid)).await
    }

    pub async fn remove(&self) -> bool {
        self.clone().delete(Database::global().connection()).await.is_ok()
    }
}
