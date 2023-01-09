use crate::database::schema::memes as MemesSchema;
use crate::database::schema::meme_likes as MemeLikesSchema;

use diesel::prelude::*;
use chrono::prelude::*;
use uuid::Uuid;
use serde_json::Value as Json;

#[derive(Debug, Selectable, Queryable, Identifiable, Insertable)]
#[diesel(primary_key(uuid))]
#[diesel(table_name = MemesSchema)]
pub struct Meme {
    pub uuid: Uuid,
    pub msg_id: Option<i64>,
    pub user_id: i64,
    pub chat_id: i64,
    pub photos: Option<Json>,
    pub posted_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Selectable, Queryable, Identifiable, Insertable, Associations)]
#[diesel(table_name = MemeLikesSchema)]
#[diesel(belongs_to(Meme, foreign_key = meme_uuid))]
#[diesel(primary_key(uuid))]
pub struct MemeLike {
    pub uuid: Uuid,
    pub meme_uuid: Uuid,
    pub user_id: i64,
    pub num: i16,
    pub created_at: Option<NaiveDateTime>,
}