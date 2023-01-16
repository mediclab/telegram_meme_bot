use serde_json::Value as Json;
use uuid::Uuid;

use crate::database::{
    manager::DBManager, models::*, schema::meme_likes as MemeLikesSchema,
    schema::memes as MemesSchema, PgPooledConnection,
};

use crate::bot::utils::*;
use diesel::{dsl, prelude::*, result::Error};

pub struct MemeRepository {
    db_manager: DBManager,
}

trait Repository {
    fn get_connection(&self) -> PgPooledConnection;
}

impl Repository for MemeRepository {
    fn get_connection(&self) -> PgPooledConnection {
        self.db_manager
            .get_pool()
            .expect("Cannot get connection from pool")
    }
}

impl MemeRepository {
    pub fn new(db_manager: DBManager) -> Self {
        Self { db_manager }
    }

    pub fn add(&self, user_id: i64, chat_id: i64, photos: Json) -> Result<Meme, Error> {
        diesel::insert_into(MemesSchema::table)
            .values((
                MemesSchema::dsl::user_id.eq(user_id),
                MemesSchema::dsl::chat_id.eq(chat_id),
                MemesSchema::dsl::photos.eq(Some(photos)),
            ))
            .get_result::<Meme>(&mut *self.get_connection())
    }

    pub fn add_msg_id(&self, uuid: &Uuid, msg_id: i64) -> bool {
        diesel::update(MemesSchema::table)
            .filter(MemesSchema::dsl::uuid.eq(uuid))
            .set(MemesSchema::dsl::msg_id.eq(msg_id))
            .execute(&mut *self.get_connection())
            .is_ok()
    }

    pub fn get(&self, uuid: &Uuid) -> Result<Meme, Error> {
        MemesSchema::table
            .find(uuid)
            .first(&mut *self.get_connection())
    }

    pub fn get_by_msg_id(&self, msg_id: i64, chat_id: i64) -> Result<Meme, Error> {
        MemesSchema::table
            .filter(MemesSchema::dsl::msg_id.eq(msg_id))
            .filter(MemesSchema::dsl::chat_id.eq(chat_id))
            .first(&mut *self.get_connection())
    }

    pub fn delete(&self, uuid: &Uuid) -> bool {
        diesel::delete(MemesSchema::table)
            .filter(MemesSchema::dsl::uuid.eq(uuid))
            .execute(&mut *self.get_connection())
            .is_ok()
    }
}

pub struct MemeLikeRepository {
    db_manager: DBManager,
}

impl Repository for MemeLikeRepository {
    fn get_connection(&self) -> PgPooledConnection {
        self.db_manager
            .get_pool()
            .expect("Cannot get connection from pool")
    }
}

impl MemeLikeRepository {
    pub fn new(db_manager: DBManager) -> Self {
        Self { db_manager }
    }

    pub fn like(&self, from_user_id: i64, uuid: &Uuid) -> bool {
        self.insert(from_user_id, uuid, MemeLikeOperation::Like)
    }

    pub fn dislike(&self, from_user_id: i64, uuid: &Uuid) -> bool {
        self.insert(from_user_id, uuid, MemeLikeOperation::Dislike)
    }

    pub fn cancel_like(&self, from_user_id: i64, uuid: &Uuid) -> bool {
        self.cancel(from_user_id, uuid, MemeLikeOperation::Like)
    }

    pub fn cancel_dislike(&self, from_user_id: i64, uuid: &Uuid) -> bool {
        self.cancel(from_user_id, uuid, MemeLikeOperation::Dislike)
    }

    pub fn like_exists(&self, from_user_id: i64, uuid: &Uuid) -> bool {
        self.exists(from_user_id, uuid, MemeLikeOperation::Like)
    }

    pub fn dislike_exists(&self, from_user_id: i64, uuid: &Uuid) -> bool {
        self.exists(from_user_id, uuid, MemeLikeOperation::Dislike)
    }

    pub fn count_likes(&self, uuid: &Uuid) -> i64 {
        self.count(uuid, MemeLikeOperation::Like)
    }

    pub fn count_dislikes(&self, uuid: &Uuid) -> i64 {
        self.count(uuid, MemeLikeOperation::Dislike)
    }

    pub fn get_top_meme(&self, period: Period) -> Result<(Meme, i64), Error> {
        use diesel::sql_types::BigInt;
        use diesel::sql_types::Bool;

        let (start, end) = period.dates();

        MemesSchema::table
            .left_join(MemeLikesSchema::table)
            .group_by((MemesSchema::dsl::uuid, MemesSchema::dsl::posted_at))
            .filter(MemesSchema::dsl::posted_at.ge(start.naive_utc()))
            .filter(MemesSchema::dsl::posted_at.le(end.naive_utc()))
            .filter(dsl::sql::<Bool>("likes IS NOT NULL"))
            .select((
                MemesSchema::all_columns,
                dsl::sql::<BigInt>("SUM(\"meme_likes\".\"num\") as likes"),
            ))
            .order_by(dsl::sql::<BigInt>("likes DESC"))
            .then_order_by(MemesSchema::dsl::posted_at.desc())
            .first::<(Meme, i64)>(&mut *self.get_connection())
    }

    fn insert(&self, from_user_id: i64, uuid: &Uuid, operation: MemeLikeOperation) -> bool {
        diesel::insert_into(MemeLikesSchema::table)
            .values((
                MemeLikesSchema::dsl::user_id.eq(from_user_id),
                MemeLikesSchema::dsl::meme_uuid.eq(uuid),
                MemeLikesSchema::dsl::num.eq(operation.id()),
            ))
            .on_conflict((
                MemeLikesSchema::dsl::user_id,
                MemeLikesSchema::dsl::meme_uuid,
            ))
            .do_update()
            .set(MemeLikesSchema::dsl::num.eq(operation.id()))
            .execute(&mut *self.get_connection())
            .is_ok()
    }

    fn cancel(&self, from_user_id: i64, uuid: &Uuid, operation: MemeLikeOperation) -> bool {
        diesel::delete(MemeLikesSchema::table)
            .filter(MemeLikesSchema::dsl::meme_uuid.eq(uuid))
            .filter(MemeLikesSchema::dsl::user_id.eq(from_user_id))
            .filter(MemeLikesSchema::dsl::num.eq(operation.id()))
            .execute(&mut *self.get_connection())
            .is_ok()
    }

    fn exists(&self, from_user_id: i64, uuid: &Uuid, operation: MemeLikeOperation) -> bool {
        dsl::select(dsl::exists(
            MemeLikesSchema::table
                .filter(MemeLikesSchema::dsl::meme_uuid.eq(uuid))
                .filter(MemeLikesSchema::dsl::user_id.eq(from_user_id))
                .filter(MemeLikesSchema::dsl::num.eq(operation.id())),
        ))
        .get_result(&mut *self.get_connection())
        .unwrap_or(false)
    }

    fn count(&self, uuid: &Uuid, operation: MemeLikeOperation) -> i64 {
        MemeLikesSchema::table
            .filter(MemeLikesSchema::dsl::meme_uuid.eq(uuid))
            .filter(MemeLikesSchema::dsl::num.eq(operation.id()))
            .count()
            .get_result(&mut *self.get_connection())
            .unwrap_or(0)
    }
}
