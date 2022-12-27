use teloxide::{prelude::*, types::{InlineKeyboardButton, InlineKeyboardMarkup}};
use std::error::Error;
use std::sync::Arc;

use crate::BotState;
use crate::database::repository::MemeLikeRepository;

pub async fn callback_handle(bot: Bot, callback: CallbackQuery, state: Arc<BotState>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let msg = callback.message.unwrap();
    let repository = MemeLikeRepository::new(state.db_manager.clone());

    match callback.data.unwrap().as_str() {
        "Like" => {
            repository.like(&msg);
        },
        "Dislike" => {
            repository.dislike(&msg);
        }
        _ => {},
    }

    let likes: i64 = repository.count_likes(&msg);
    let dislikes: i64 = repository.count_dislikes(&msg);

    let _ = bot
        .edit_message_reply_markup(msg.chat.id, msg.id)
        .reply_markup(self::get_likes_markup(likes, dislikes))
        .await?
    ;

    Ok(())
}

fn get_likes_markup(likes: i64, dislikes: i64) -> InlineKeyboardMarkup {
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