use crate::schema::memes as MemesSchema;
use crate::schema::meme_likes as MemeLikesSchema;
use diesel::prelude::*;
use std::time::SystemTime;

#[derive(Debug, Selectable, Queryable, Identifiable, Insertable)]
#[diesel(primary_key(msg_id))]
#[diesel(table_name = MemesSchema)]
pub struct Meme {
    pub msg_id: i64,
    pub bot_msg_id: i64,
    pub user_id: i64,
    pub chat_id: i64,
    pub photos: Option<serde_json::Value>,
    pub posted_at: Option<SystemTime>,
    pub updated_at: Option<SystemTime>
}

#[derive(Debug, Selectable, Queryable, Insertable, Identifiable, Associations)]
#[diesel(table_name = MemeLikesSchema)]
#[diesel(belongs_to(Meme, foreign_key = msg_id))]
pub struct MemeLike {
    pub id: i64,
    pub user_id: i64,
    pub msg_id: i64,
    pub num: i16,
    pub created_at: Option<SystemTime>,
}