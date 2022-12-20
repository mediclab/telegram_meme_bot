extern crate derive_builder;

use crate::db::PgPooledConnection;
use diesel::prelude::*;
use std::time::SystemTime;

#[derive(Builder, Debug, Selectable, Queryable, Identifiable, Insertable)]
#[diesel(table_name = crate::schema::memes)]
pub struct Memes {
    pub id: i64,
    pub user_id: i64,
    pub chat_id: i64,
    pub photos: Option<serde_json::Value>,
    pub posted_at: Option<SystemTime>,
    pub updated_at: Option<SystemTime>,
}

impl Memes {
    pub fn add(conn: &mut PgPooledConnection, userId: i64, chatId: i64, photo: Option<serde_json::Value>) {
        let meme_id = diesel::insert_into(crate::schema::memes::table)
            .values(
                (
                    crate::schema::memes::dsl::user_id.eq(userId),
                    crate::schema::memes::dsl::chat_id.eq(chatId),
                    crate::schema::memes::dsl::photos.eq(photo)
                )
            )
            .returning(crate::schema::memes::dsl::id)
            .execute(&mut *conn)
            .expect("Can't save new meme")
        ;

        diesel::insert_into(crate::schema::meme_likes::table)
            .values(
                (
                    crate::schema::meme_likes::dsl::user_id.eq(userId),
                    crate::schema::meme_likes::dsl::meme_id.eq(meme_id as i64),
                    crate::schema::meme_likes::dsl::num.eq(0)
                )
            )
            .execute(&mut *conn)
            .expect("Can't save new meme like")
        ;
    }
}

#[derive(Debug, Queryable, Insertable, Associations)]
#[diesel(table_name = crate::schema::meme_likes)]
#[diesel(belongs_to(Memes))]
pub struct Likes {
    pub user_id: i64,
    pub meme_id: i64,
    pub num: i16,
}

impl Likes {

}