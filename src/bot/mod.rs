use teloxide::adaptors::DefaultParseMode;
use teloxide::prelude::{ChatId, Requester, UserId};
use teloxide::types::User;

pub mod callbacks;
pub mod commands;
pub mod markups;
pub mod messages;
pub mod statistics;

pub type Bot = DefaultParseMode<teloxide::Bot>;

pub async fn get_chat_user(bot: &Bot, chat_id: i64, user_id: i64) -> User {
    let member = bot
        .get_chat_member(ChatId(chat_id), UserId(user_id as u64))
        .await
        .expect("Can't get chat member");

    member.user
}
