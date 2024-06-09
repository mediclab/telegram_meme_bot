use crate::database::Database;
use sea_orm::entity::prelude::*;
use sea_orm::sea_query::OnConflict;
use sea_orm::{FromQueryResult, QuerySelect, QueryTrait, Set};

#[derive(DeriveIden)]
pub enum MemeLikes {
    Table,
}

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
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::UserId",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Users,
}

impl Related<super::memes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Memes.def()
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(FromQueryResult, Debug, Clone)]
pub struct MemeLikesCountAll {
    pub likes: i64,
    pub dislikes: i64,
}

pub enum MemeLikeOperation {
    Like,
    Dislike,
}

impl MemeLikeOperation {
    pub fn id(&self) -> i16 {
        match *self {
            MemeLikeOperation::Like => 1,
            MemeLikeOperation::Dislike => -1,
        }
    }
}

impl Entity {
    pub async fn add(meme_uuid: Uuid, user_id: i64, operation: MemeLikeOperation) -> bool {
        Entity::insert(ActiveModel {
            meme_uuid: Set(Some(meme_uuid)),
            user_id: Set(user_id),
            num: Set(operation.id()),
            ..Default::default()
        })
        .on_conflict(
            OnConflict::columns([Column::UserId, Column::MemeUuid])
                .update_column(Column::Num)
                .to_owned(),
        )
        .exec(Database::global().connection())
        .await
        .is_ok()
    }

    pub async fn exists(meme_uuid: Uuid, user_id: i64, operation: MemeLikeOperation) -> bool {
        let query_res: Option<i64> = Entity::find()
            .filter(Column::MemeUuid.eq(meme_uuid))
            .filter(Column::UserId.eq(user_id))
            .filter(Column::Num.eq(operation.id()))
            .select_only()
            .column_as(Column::Uuid.count(), "count")
            .into_tuple()
            .one(Database::global().connection())
            .await
            .expect("Can't execute exists statement");

        if let Some(count) = query_res {
            return count > 0;
        }

        false
    }

    pub async fn remove(meme_uuid: Uuid, user_id: i64, operation: MemeLikeOperation) -> bool {
        Entity::delete_many()
            .filter(Column::MemeUuid.eq(meme_uuid))
            .filter(Column::UserId.eq(user_id))
            .filter(Column::Num.eq(operation.id()))
            .exec(Database::global().connection())
            .await
            .is_ok()
    }

    pub async fn count_all(meme_uuid: Option<Uuid>) -> Option<MemeLikesCountAll> {
        let res = Entity::find()
            .apply_if(meme_uuid, |query, v| query.filter(Column::MemeUuid.eq(v)))
            .select_only()
            .column_as(
                Expr::cust(r#"COUNT("meme_likes"."num") FILTER (WHERE "meme_likes"."num" = 1)"#),
                "likes",
            )
            .column_as(
                Expr::cust(r#"COUNT("meme_likes"."num") FILTER (WHERE "meme_likes"."num" = -1)"#),
                "dislikes",
            )
            .into_model::<MemeLikesCountAll>()
            .one(Database::global().connection())
            .await;

        res.unwrap_or_else(|e| {
            error!("Can't get meme from database: {e}");
            None
        })
    }
}
