use teloxide::{prelude::*, types::{InputFile, ReplyMarkup}};
use std::error::Error;
use std::sync::Arc;
use crate::bot::markups::*;

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
            let markup = MemeMarkup::new(0, 0, meme.uuid);
            let bot_msg = bot.send_photo(msg.chat.id, InputFile::file_id(&photos[0].file.id))
                .caption(format!("Оцените мем {}", user_text))
                .reply_markup(ReplyMarkup::InlineKeyboard(markup.get_markup())).await?
            ;

            repository.add_msg_id(&meme.uuid, &bot_msg);
        },
        None => {}
    }
    
    Ok(())
}