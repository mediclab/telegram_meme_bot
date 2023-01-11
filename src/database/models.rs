use crate::database::schema::meme_likes as MemeLikesSchema;
use crate::database::schema::memes as MemesSchema;

use chrono::prelude::*;
use diesel::prelude::*;
use serde_json::Value as Json;
use teloxide::types::{ChatId, MessageId, UserId};
use uuid::Uuid;

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

impl Meme {
    pub fn chat_id(&self) -> ChatId {
        ChatId(self.chat_id)
    }

    pub fn user_id(&self) -> UserId {
        UserId(self.user_id as u64)
    }

    pub fn msg_id(&self) -> MessageId {
        MessageId(self.msg_id.unwrap() as i32)
    }
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
