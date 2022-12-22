extern crate derive_builder;

use crate::db::PgPooledConnection;
use diesel::prelude::*;
use std::time::SystemTime;
use diesel::result::Error;
use diesel::dsl;

use crate::schema::memes as MemesSchema;
use crate::schema::meme_likes as MemeLikesSchema;

#[derive(Builder, Debug, Selectable, Queryable, Identifiable, Insertable)]
#[diesel(table_name = crate::schema::memes)]
pub struct Meme {
    pub id: i64,
    pub user_id: i64,
    pub chat_id: i64,
    pub photos: Option<serde_json::Value>,
    pub posted_at: Option<SystemTime>,
    pub updated_at: Option<SystemTime>,
}

impl Meme {
    pub fn add(conn: &mut PgPooledConnection, user_id: i64, chat_id: i64, photo: Option<serde_json::Value>) -> Result<Meme, Error> {
        diesel::insert_into(MemesSchema::table)
            .values(
                (
                    MemesSchema::dsl::user_id.eq(user_id),
                    MemesSchema::dsl::chat_id.eq(chat_id),
                    MemesSchema::dsl::photos.eq(photo)
                )
            )
            .get_result::<Meme>(&mut *conn)
    }
}

#[derive(Debug, Selectable, Queryable, Insertable, Identifiable, Associations)]
#[diesel(table_name = MemeLikesSchema)]
#[diesel(belongs_to(Meme))]
pub struct MemeLike {
    pub id: i64,
    pub user_id: i64,
    pub meme_id: i64,
    pub num: i16,
    pub created_at: Option<SystemTime>,
}

impl MemeLike {
    pub fn like(conn: &mut PgPooledConnection, user_id: i64, meme_id: i64) -> bool {
        if MemeLike::exists(conn, user_id, meme_id) {
            diesel::update(MemeLikesSchema::table)
                .filter(MemeLikesSchema::dsl::user_id.eq(user_id))
                .filter(MemeLikesSchema::dsl::meme_id.eq(meme_id))
                .set(MemeLikesSchema::dsl::num.eq(1))
                .execute(&mut *conn).is_ok()
        } else {
            diesel::insert_into(MemeLikesSchema::table)
            .values(
                (
                    MemeLikesSchema::dsl::user_id.eq(user_id),
                    MemeLikesSchema::dsl::meme_id.eq(meme_id),
                    MemeLikesSchema::dsl::num.eq(1)
                )
            )
            .execute(&mut *conn).is_ok()
        }
    }

    pub fn dislike(conn: &mut PgPooledConnection, user_id: i64, meme_id: i64) -> bool {
        if MemeLike::exists(conn, user_id, meme_id) {
            diesel::update(MemeLikesSchema::table)
                .filter(MemeLikesSchema::dsl::user_id.eq(user_id))
                .filter(MemeLikesSchema::dsl::meme_id.eq(meme_id))
                .set(MemeLikesSchema::dsl::num.eq(-1))
                .execute(&mut *conn).is_ok()
        } else {
            diesel::insert_into(MemeLikesSchema::table)
            .values(
                (
                    MemeLikesSchema::dsl::user_id.eq(user_id),
                    MemeLikesSchema::dsl::meme_id.eq(meme_id),
                    MemeLikesSchema::dsl::num.eq(-1)
                )
            )
            .execute(&mut *conn).is_ok()
        }
    }

    pub fn exists(conn: &mut PgPooledConnection, user_id: i64, meme_id: i64) -> bool {
        dsl::select(dsl::exists(
            MemeLikesSchema::table
            .filter(
                MemeLikesSchema::dsl::meme_id.eq(meme_id)
            ).filter(
                MemeLikesSchema::dsl::user_id.eq(user_id)
            )
        )).get_result(&mut *conn).unwrap_or(false)
    }

    pub fn count_likes(conn: &mut PgPooledConnection, user_id: i64, meme_id: i64) -> i64 {
        MemeLikesSchema::table
            .filter(MemeLikesSchema::dsl::meme_id.eq(meme_id))
            .filter(MemeLikesSchema::dsl::user_id.eq(user_id))
            .filter(MemeLikesSchema::dsl::num.eq(1))
            .count()
            .get_result(&mut *conn).unwrap_or(0)
    }

    pub fn count_dislikes(conn: &mut PgPooledConnection, user_id: i64, meme_id: i64) -> i64 {
        MemeLikesSchema::table
            .filter(MemeLikesSchema::dsl::meme_id.eq(meme_id))
            .filter(MemeLikesSchema::dsl::user_id.eq(user_id))
            .filter(MemeLikesSchema::dsl::num.eq(-1))
            .count()
            .get_result(&mut *conn).unwrap_or(0)
    }
}