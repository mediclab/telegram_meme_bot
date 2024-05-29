use crate::database::entity::{chats, memes, users};
use crate::database::Database;
use sea_orm::ActiveModelTrait;
use sea_orm::Set;
use teloxide::prelude::{ChatId, Message, UserId};
use teloxide::types::{Chat, MessageId, User};

impl From<User> for users::ActiveModel {
    fn from(value: User) -> Self {
        users::ActiveModel {
            user_id: Set(value.id.0 as i64),
            username: Set(value.username),
            firstname: Set(value.first_name),
            lastname: Set(value.last_name),
            ..Default::default()
        }
    }
}

impl From<Chat> for chats::ActiveModel {
    fn from(value: Chat) -> Self {
        Self {
            chat_id: Set(value.id.0),
            chatname: Set(value.username().map(|d| d.to_string())),
            description: Set(value.description().map(|d| d.to_string())),
            title: Set(value.title().map(|d| d.to_string())),
            ..Default::default()
        }
    }
}

impl memes::Entity {
    pub async fn add(message: &Message, l_hash: &Option<String>, s_hash: &Option<String>) -> Option<memes::Model> {
        let json = if message.photo().is_some() {
            Option::from(serde_json::json!(message.photo()))
        } else if message.video().is_some() {
            Option::from(serde_json::json!(message.video()))
        } else {
            None
        };

        let res = memes::ActiveModel {
            msg_id: sea_orm::Set(Some(message.id.0 as i64)),
            user_id: sea_orm::Set(message.from().unwrap().id.0 as i64),
            chat_id: sea_orm::Set(message.chat.id.0),
            photos: sea_orm::Set(json),
            long_hash: sea_orm::Set(l_hash.clone()),
            short_hash: sea_orm::Set(s_hash.clone()),
            ..Default::default()
        }
        .insert(Database::global().connection())
        .await;

        match res {
            Ok(m) => Some(m),
            Err(e) => {
                error!("Can't add meme to database: {e}");
                None
            }
        }
    }
}

impl memes::Model {
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

impl users::Entity {
    pub async fn add(tg_user: &User) -> Option<users::Model> {
        let res = users::ActiveModel::from(tg_user.clone())
            .insert(Database::global().connection())
            .await;

        match res {
            Ok(m) => Some(m),
            Err(e) => {
                error!("Can't add user to database: {e}");
                None
            }
        }
    }
}

impl chats::Entity {
    pub async fn add(tg_chat: &Chat) -> Option<chats::Model> {
        let res = chats::ActiveModel::from(tg_chat.clone())
            .insert(Database::global().connection())
            .await;

        match res {
            Ok(c) => Some(c),
            Err(e) => {
                error!("Can't add user to database: {e}");
                None
            }
        }
    }
}
