use teloxide::{prelude::*, types::{InputFile, ReplyMarkup, InlineKeyboardButton, InlineKeyboardMarkup}};
use std::error::Error;
use std::sync::Arc;

use crate::BotState;
use crate::database::repository::MemeRepository;

pub async fn message_handle(bot: Bot, msg: Message, state: Arc<BotState>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let user = msg.from().unwrap();
    let repository = MemeRepository::new(state.db_manager.clone());

    let user_text = match &user.username {
        Some(uname) => format!("@{}", uname),
        None => format!("[{}](tg://user?id={})", user.first_name, user.id.0)
    };

    match msg.photo() {
        Some(photos) => {
            let meme = repository.add(&msg).unwrap();

            bot.delete_message(msg.chat.id, msg.id).await?;
            let bot_msg = bot.send_photo(msg.chat.id, InputFile::file_id(&photos[0].file.id))
                .caption(format!("Оцените мем {}", user_text))
                .reply_markup(ReplyMarkup::InlineKeyboard(
                    self::get_likes_markup(0, 0)
                )).await?
            ;

            repository.add_msg_id(&meme.uuid, &bot_msg);
        },
        None => {}
    }
    
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