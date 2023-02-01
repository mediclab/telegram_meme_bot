use crate::database::schema::{
    chats as ChatsSchema, meme_likes as MemeLikesSchema, memes as MemesSchema, users as UsersSchema,
};
use chrono::prelude::*;
use diesel::prelude::*;
use serde_json::Value as Json;
use teloxide::types::{Chat as TgChat, ChatId, Message, MessageId, User as TgUser, UserId};
use uuid::Uuid;

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

#[derive(Debug, Insertable)]
#[diesel(table_name = MemesSchema)]
pub struct AddMeme {
    pub user_id: i64,
    pub chat_id: i64,
    pub photos: Option<Json>,
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

impl AddMeme {
    pub fn new_from_tg(msg: &Message, l_hash: &Option<String>, s_hash: &Option<String>) -> Self {
        let json = if msg.photo().is_some() {
            Option::from(serde_json::json!(msg.photo()))
        } else if msg.video().is_some() {
            Option::from(serde_json::json!(msg.video()))
        } else {
            None
        };

        Self {
            user_id: msg.from().unwrap().id.0 as i64,
            chat_id: msg.chat.id.0,
            photos: json,
            long_hash: l_hash.clone(),
            short_hash: s_hash.clone(),
        }
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

#[derive(Debug, Insertable)]
#[diesel(table_name = UsersSchema)]
pub struct AddUser {
    pub user_id: i64,
    pub username: Option<String>,
    pub firstname: String,
    pub lastname: Option<String>,
}

impl AddUser {
    pub fn new_from_tg(user: &TgUser) -> Self {
        let u = user.clone();

        Self {
            user_id: u.id.0 as i64,
            username: u.username,
            firstname: u.first_name,
            lastname: u.last_name,
        }
    }
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

#[derive(Debug, Insertable)]
#[diesel(table_name = ChatsSchema)]
pub struct AddChat {
    pub chat_id: i64,
    pub chatname: Option<String>,
    pub description: Option<String>,
    pub title: Option<String>,
}

impl AddChat {
    pub fn new_from_tg(chat: &TgChat) -> Self {
        Self {
            chat_id: chat.id.0,
            chatname: chat.username().map(|d| d.to_string()),
            description: chat.description().map(|d| d.to_string()),
            title: chat.title().map(|d| d.to_string()),
        }
    }
}
