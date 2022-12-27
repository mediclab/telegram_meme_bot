use crate::database::schema::memes as MemesSchema;
use crate::database::schema::meme_likes as MemeLikesSchema;

use diesel::prelude::*;
use std::time::SystemTime;
use uuid::Uuid;

#[derive(Debug, Selectable, Queryable, Identifiable, Insertable)]
#[diesel(primary_key(uuid))]
#[diesel(table_name = MemesSchema)]
pub struct Meme {
    pub uuid: Uuid,
    pub msg_id: Option<i64>,
    pub user_id: i64,
    pub chat_id: i64,
    //pub hash: Option<String>,
    pub photos: Option<serde_json::Value>,
    pub posted_at: Option<SystemTime>,
    pub updated_at: Option<SystemTime>
}

#[derive(Debug, Selectable, Queryable, Identifiable, Insertable)]
#[diesel(table_name = MemeLikesSchema)]
#[diesel(primary_key(uuid))]
pub struct MemeLike {
    pub uuid: Uuid,
    pub user_id: i64,
    pub msg_id: i64,
    pub num: i16,
    pub created_at: Option<SystemTime>,
}