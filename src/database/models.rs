use chrono::prelude::*;
use diesel::prelude::*;
use serde_json::Value as Json;
use teloxide::types::{ChatId, MessageId, UserId};
use uuid::Uuid;

use crate::database::schema::{
    chats as ChatsSchema, meme_likes as MemeLikesSchema, memes as MemesSchema, users as UsersSchema,
};

#[derive(Debug, Selectable, Queryable, Identifiable)]
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
    pub long_hash: Option<String>,
    pub short_hash: Option<String>,
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

#[derive(Debug, Selectable, Queryable, Identifiable, Insertable)]
#[diesel(table_name = UsersSchema)]
#[diesel(primary_key(user_id))]
pub struct User {
    pub user_id: i64,
    pub username: Option<String>,
    pub firstname: String,
    pub lastname: Option<String>,
    pub deleted_at: Option<NaiveDateTime>,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Selectable, Queryable, Identifiable)]
#[diesel(table_name = ChatsSchema)]
#[diesel(primary_key(chat_id))]
pub struct Chat {
    pub chat_id: i64,
    pub chatname: Option<String>,
    pub description: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub title: Option<String>,
    pub deleted_at: Option<NaiveDateTime>,
}
