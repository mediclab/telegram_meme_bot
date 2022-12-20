use serde_json::json;
use teloxide::{prelude::*, types::{InputFile, ReplyMarkup, InlineKeyboardButton, InlineKeyboardMarkup}};
use std::error::Error;
use redis::Commands;
use std::sync::Arc;

use crate::models::*;
use crate::BotState;

pub async fn message_handle(bot: Bot, msg: Message, state: Arc<BotState>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let user = msg.from().unwrap();
    let conn = &mut state.db_manager.get_pool().unwrap();

    let user_text = match &user.username {
        Some(uname) => format!("@{}", uname),
        None => format!("[{}](tg://user?id={})", user.first_name, user.id.0)
    };

    match msg.photo() {
        Some(photos) => {
            Memes::add(conn, user.id.0 as i64, msg.chat.id.0, Some(json!(photos)));

            bot.delete_message(msg.chat.id, msg.id).await?;
            bot.send_photo(msg.chat.id, InputFile::file_id(&photos[0].file.id))
            .caption(format!("Оцените мем {}", user_text))
            .reply_markup(ReplyMarkup::InlineKeyboard(
                self::get_likes_markup(0, 0)
            )).await?;
        },
        None => {}
    }
    
    Ok(())
}

pub async fn callback_handle(bot: Bot, callback: CallbackQuery, state: Arc<BotState>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let msg = callback.message.unwrap();
    let mut redis = state.redis.get_connection().expect("Connection error");

    let mut likes: i32 = redis.get(format!("likes_{}", msg.id.0)).unwrap_or(0);
    let mut dislikes: i32 = redis.get(format!("dislikes_{}", msg.id.0)).unwrap_or(0);

    match callback.data.unwrap().as_str() {
        "Like" => {
            likes = redis.incr(format!("likes_{}", msg.id.0), 1)?;
        },
        "Dislike" => {
            dislikes = redis.incr(format!("dislikes_{}", msg.id.0), 1)?;
        }
        _ => {},
    }

    let _ = bot
        .edit_message_reply_markup(msg.chat.id, msg.id)
        .reply_markup(self::get_likes_markup(likes, dislikes))
        .await?
    ;

    Ok(())
}

fn get_likes_markup(likes: i32, dislikes: i32) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(
        vec![vec![
            InlineKeyboardButton::callback(
                format!(
                    "{} Like ({})", String::from(emojis::get_by_shortcode("heart").unwrap().as_str()),
                    likes
                ),
                String::from("Like")
            ),
            InlineKeyboardButton::callback(
                format!(
                    "{} Dislike ({})",String::from(emojis::get_by_shortcode("broken_heart").unwrap().as_str()),
                    dislikes
                ),
                String::from("Dislike")
            )
        ]]
    )
}