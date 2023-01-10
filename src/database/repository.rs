use serde_json::Value as Json;
use uuid::Uuid;
use now::DateTimeNow;
use chrono::prelude::*;

use crate::database::{
    models::*,
    manager::DBManager,
    PgPooledConnection,
    schema::memes as MemesSchema,
    schema::meme_likes as MemeLikesSchema,
};

use diesel::{result::Error, dsl, prelude::*};

pub struct MemeRepository {
    db_manager: DBManager,
}

trait Repository {
    fn get_connection(&self) -> PgPooledConnection;
}

impl Repository for MemeRepository {
    fn get_connection(&self) -> PgPooledConnection {
        self.db_manager.get_pool().expect("Cannot get connection from pool")
    }
}

impl MemeRepository {
    pub fn new(db_manager: DBManager) -> Self {
        Self { db_manager }
    }

    pub fn add(&self, user_id: i64, chat_id: i64, photos: Json) -> Result<Meme, Error> {
        diesel::insert_into(MemesSchema::table)
            .values(
                (
                    MemesSchema::dsl::user_id.eq(user_id),
                    MemesSchema::dsl::chat_id.eq(chat_id),
                    MemesSchema::dsl::photos.eq(Some(photos)),
                )
            )
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
        MemesSchema::table.find(uuid).first(&mut *self.get_connection())
    }

    pub fn get_by_msg_id(&self, msg_id: i64, chat_id: i64) -> Result<Meme, Error> {
        MemesSchema::table
            .filter(MemesSchema::dsl::msg_id.eq(msg_id))
            .filter(MemesSchema::dsl::chat_id.eq(chat_id))
            .first(&mut *self.get_connection())
    }
}

pub struct MemeLikeRepository {
    db_manager: DBManager,
}

impl Repository for MemeLikeRepository {
    fn get_connection(&self) -> PgPooledConnection {
        self.db_manager.get_pool().expect("Cannot get connection from pool")
    }
}

impl MemeLikeRepository {
    pub fn new(db_manager: DBManager) -> Self {
        Self { db_manager }
    }

    pub fn like(&self, from_user_id: i64, uuid: &Uuid) -> bool {
        if self.exists(from_user_id, uuid) {
            diesel::update(MemeLikesSchema::table)
                .filter(MemeLikesSchema::dsl::user_id.eq(from_user_id))
                .filter(MemeLikesSchema::dsl::meme_uuid.eq(uuid))
                .set(MemeLikesSchema::dsl::num.eq(1))
                .execute(&mut *self.get_connection())
                .is_ok()
        } else {
            diesel::insert_into(MemeLikesSchema::table)
                .values(
                    (
                        MemeLikesSchema::dsl::user_id.eq(from_user_id),
                        MemeLikesSchema::dsl::meme_uuid.eq(uuid),
                        MemeLikesSchema::dsl::num.eq(1)
                    )
                )
                .execute(&mut *self.get_connection())
                .is_ok()
        }
    }

    pub fn dislike(&self, from_user_id: i64, uuid: &Uuid) -> bool {
        if self.exists(from_user_id, uuid) {
            diesel::update(MemeLikesSchema::table)
                .filter(MemeLikesSchema::dsl::user_id.eq(from_user_id))
                .filter(MemeLikesSchema::dsl::meme_uuid.eq(uuid))
                .set(MemeLikesSchema::dsl::num.eq(-1))
                .execute(&mut *self.get_connection())
                .is_ok()
        } else {
            diesel::insert_into(MemeLikesSchema::table)
                .values(
                    (
                        MemeLikesSchema::dsl::user_id.eq(from_user_id),
                        MemeLikesSchema::dsl::meme_uuid.eq(uuid),
                        MemeLikesSchema::dsl::num.eq(-1)
                    )
                )
                .execute(&mut *self.get_connection())
                .is_ok()
        }
    }

    pub fn exists(&self, from_user_id: i64, uuid: &Uuid) -> bool {
        dsl::select(dsl::exists(
            MemeLikesSchema::table
                .filter(MemeLikesSchema::dsl::meme_uuid.eq(uuid))
                .filter(MemeLikesSchema::dsl::user_id.eq(from_user_id))
        )).get_result(&mut *self.get_connection())
            .unwrap_or(false)
    }

    pub fn count_likes(&self, uuid: &Uuid) -> i64 {
        MemeLikesSchema::table
            .filter(MemeLikesSchema::dsl::meme_uuid.eq(uuid))
            .filter(MemeLikesSchema::dsl::num.eq(1))
            .count()
            .get_result(&mut *self.get_connection())
            .unwrap_or(0)
    }

    pub fn count_dislikes(&self, uuid: &Uuid) -> i64 {
        MemeLikesSchema::table
            .filter(MemeLikesSchema::dsl::meme_uuid.eq(uuid))
            .filter(MemeLikesSchema::dsl::num.eq(-1))
            .count()
            .get_result(&mut *self.get_connection())
            .unwrap_or(0)
    }

    pub fn meme_of_week(&self) -> Result<(Meme, i64), Error> {
        self.get_top_meme(
            Utc::now().beginning_of_week().naive_utc(),
            Utc::now().end_of_week().naive_utc(),
        )
    }

    pub fn meme_of_month(&self) -> Result<(Meme, i64), Error> {
        self.get_top_meme(
            Utc::now().beginning_of_month().naive_utc(),
            Utc::now().end_of_month().naive_utc(),
        )
    }

    pub fn meme_of_year(&self) -> Result<(Meme, i64), Error> {
        self.get_top_meme(
            Utc::now().beginning_of_year().naive_utc(),
            Utc::now().end_of_year().naive_utc(),
        )
    }

    fn get_top_meme(&self, start: NaiveDateTime, end: NaiveDateTime) -> Result<(Meme, i64), Error> {
        use diesel::sql_types::BigInt;

        MemesSchema::table
            .left_join(MemeLikesSchema::table)
            .group_by((MemesSchema::dsl::uuid, MemesSchema::dsl::posted_at))
            .filter(MemeLikesSchema::dsl::created_at.ge(start))
            .filter(MemeLikesSchema::dsl::created_at.le(end))
            .select((MemesSchema::all_columns, dsl::sql::<BigInt>("SUM(\"meme_likes\".\"num\") as likes")))
            .order_by(dsl::sql::<BigInt>("likes DESC"))
            .then_order_by(MemesSchema::dsl::posted_at.desc())
            .first::<(Meme, i64)>(&mut *self.get_connection())
    }
}