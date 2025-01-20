use crate::database::entity::meme_likes::MemeLikeOperation;
use crate::database::Database;
use chrono::Utc;
use sea_orm::entity::prelude::*;
use sea_orm::sea_query::Alias;
use sea_orm::{sea_query::OnConflict, FromQueryResult, JoinType, Order, QueryOrder, QuerySelect, Set};

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
pub enum Relation {
    #[sea_orm(has_many = "super::meme_likes::Entity")]
    MemeLikes,
    #[sea_orm(has_many = "super::memes::Entity")]
    Memes,
}

impl Related<super::meme_likes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MemeLikes.def()
    }
}

impl Related<super::memes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Memes.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(FromQueryResult, Debug, Clone)]
pub struct TopUser {
    pub user_id: i64,
    pub count: i64,
}

impl Entity {
    pub async fn add(model: ActiveModel) -> bool {
        Entity::insert(model)
            .on_conflict(
                OnConflict::column(Column::UserId)
                    .value(Column::DeletedAt, Expr::val(None::<DateTime>))
                    .to_owned(),
            )
            .exec(Database::global().connection())
            .await
            .is_ok()
    }

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

    pub async fn top_selfliker(from: DateTimeUtc, to: DateTimeUtc) -> Option<TopUser> {
        let res = super::memes::Entity::find()
            .select_only()
            .join(JoinType::InnerJoin, super::memes::Relation::MemeLikes.def())
            .filter(super::meme_likes::Column::CreatedAt.gte(from))
            .filter(super::meme_likes::Column::CreatedAt.lte(to))
            .filter(
                Expr::col((super::memes::Memes::Table, super::memes::Column::UserId))
                    .equals((super::meme_likes::MemeLikes::Table, super::meme_likes::Column::UserId)),
            )
            .filter(super::meme_likes::Column::Num.eq(MemeLikeOperation::Like.id()))
            .group_by(super::memes::Column::UserId)
            .column(super::memes::Column::UserId)
            .column_as(super::meme_likes::Column::Num.sum(), "count")
            .having(Expr::expr(super::meme_likes::Column::Num.sum()).gt(0))
            .order_by(Expr::col(Alias::new("count")), Order::Desc)
            .into_model::<TopUser>()
            .one(Database::global().connection())
            .await;

        res.unwrap_or_else(|e| {
            error!("Can't get top selfliker from database: {e}");
            None
        })
    }

    pub async fn top_memesender(from: DateTimeUtc, to: DateTimeUtc) -> Option<TopUser> {
        let res = Entity::find()
            .select_only()
            .join(JoinType::InnerJoin, Relation::Memes.def())
            .filter(super::memes::Column::PostedAt.gte(from))
            .filter(super::memes::Column::PostedAt.lte(to))
            .group_by(Column::UserId)
            .column(Column::UserId)
            .column_as(super::memes::Column::Uuid.count(), "count")
            .having(Expr::expr(super::memes::Column::Uuid.count()).gt(0))
            .order_by(Expr::col(Alias::new("count")), Order::Desc)
            .into_model::<TopUser>()
            .one(Database::global().connection())
            .await;

        res.unwrap_or_else(|e| {
            error!("Can't get top memesender from database: {e}");
            None
        })
    }

    pub async fn top_liker(from: DateTimeUtc, to: DateTimeUtc) -> Option<TopUser> {
        Self::top_operationist(from, to, MemeLikeOperation::Like).await
    }

    pub async fn top_disliker(from: DateTimeUtc, to: DateTimeUtc) -> Option<TopUser> {
        Self::top_operationist(from, to, MemeLikeOperation::Dislike).await
    }

    async fn top_operationist(from: DateTimeUtc, to: DateTimeUtc, operation: MemeLikeOperation) -> Option<TopUser> {
        let res = Entity::find()
            .select_only()
            .join(JoinType::InnerJoin, Relation::MemeLikes.def())
            .filter(super::meme_likes::Column::CreatedAt.gte(from))
            .filter(super::meme_likes::Column::CreatedAt.lte(to))
            .filter(super::meme_likes::Column::Num.eq(operation.id()))
            .group_by(Column::UserId)
            .column(Column::UserId)
            .column_as(super::meme_likes::Column::Num.count(), "count")
            .having(Expr::expr(super::meme_likes::Column::Num.count()).gt(0))
            .order_by(Expr::col(Alias::new("count")), Order::Desc)
            .into_model::<TopUser>()
            .one(Database::global().connection())
            .await;

        res.unwrap_or_else(|e| {
            error!("Can't get top operationist from database: {e}");
            None
        })
    }
}
