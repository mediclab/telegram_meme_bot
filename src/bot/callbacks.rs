use std::error::Error;
use std::sync::Arc;

use crate::bot::markups::*;
use crate::database::models::Meme;

use teloxide::types::MessageId;
use teloxide::{
    prelude::*,
    payloads,
    requests::JsonRequest,
};

use crate::BotState;
use crate::database::repository::*;

pub async fn callback_handle(bot: Bot, callback: CallbackQuery, state: Arc<BotState>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let repository = MemeLikeRepository::new(state.db_manager.clone());
    let data: MemeCallback = serde_json::from_str(
        &callback.data.clone().unwrap_or(r#"{}"#.to_string())
    )?;
    let meme = MemeRepository::new(state.db_manager.clone())
        .get(&data.uuid)
        .unwrap()
    ;
    let msg = callback.message.clone().unwrap();

    match data.op {
        CallbackOperations::Like => {
            operation_like(&callback, &meme, &bot, &repository).await?;
        },
        CallbackOperations::Dislike => {
            operation_dislike(&callback, &meme, &bot, &repository).await?;
        },
        CallbackOperations::Delete => {
            if meme.user_id != callback.from.id.0 as i64 {
                Err("Meme user is not a callback user")?
            }

            bot.delete_message(msg.chat.id, msg.id).await?;
            bot.delete_message(msg.chat.id, MessageId { 0: meme.msg_id.unwrap() as i32}).await?;
        },
        CallbackOperations::None => {
            if meme.user_id != callback.from.id.0 as i64 {
                Err("Meme user is not a callback user")?
            }

            bot.delete_message(msg.chat.id, msg.id).await?;
        },
    }

    Ok(())
}

fn get_likes(repository: &MemeLikeRepository, meme: &Meme) -> (i64, i64) {
    (repository.count_likes(&meme.uuid), repository.count_dislikes(&meme.uuid))
}

fn send_reply(meme: &Meme, msg: &Message, counts: (i64, i64), bot: &Bot) -> JsonRequest<payloads::EditMessageReplyMarkup> {
    let (likes, dislikes) = counts;
    let meme_markup = MemeMarkup::new(likes, dislikes, meme.uuid);

    bot
        .edit_message_reply_markup(msg.chat.id, msg.id)
        .reply_markup(meme_markup.get_markup())
}

fn operation_like(callback: &CallbackQuery, meme: &Meme, bot: &Bot, repository: &MemeLikeRepository) -> JsonRequest<payloads::EditMessageReplyMarkup> {
    let msg = callback.message.clone().unwrap();

    repository.like(&callback.from, &meme.uuid);
    send_reply(meme, &msg, get_likes(&repository, &meme), bot)
}

fn operation_dislike(callback: &CallbackQuery, meme: &Meme, bot: &Bot, repository: &MemeLikeRepository) -> JsonRequest<payloads::EditMessageReplyMarkup> {
    let msg = callback.message.clone().unwrap();

    repository.dislike(&callback.from, &meme.uuid);
    send_reply(meme, &msg, get_likes(&repository, &meme), bot)
}