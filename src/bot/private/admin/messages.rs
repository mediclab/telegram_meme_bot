use crate::bot::private::admin::types::CallbackOperations;
use crate::bot::{Bot, BotDialogue, State};
use serde_json::json;
use teloxide::{
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, InputFile, MessageKind},
};

pub async fn add_message_handle(bot: Bot, msg: Message, dialogue: BotDialogue) -> anyhow::Result<()> {
    match msg.kind {
        MessageKind::Common(_) => {
            bot.delete_message(msg.chat.id, msg.id).await?;
        }
        _ => return Ok(()),
    }

    if let Some(text) = msg.text() {
        bot.send_message(
            msg.chat.id,
            format!("Вы хотите добавить этот текст:\n\n<b>{}</b>", text),
        )
        .reply_markup(get_markup())
        .await?;
    }

    if let Some(photos) = msg.photo() {
        bot.send_photo(msg.chat.id, InputFile::file_id(&photos[0].file.id))
            .caption("Вы хотите добавить это фото?")
            .reply_markup(get_markup())
            .await?;
    }

    if let Some(doc) = msg.document() {
        bot.send_document(msg.chat.id, InputFile::file_id(&doc.file.id))
            .caption("Вы хотите добавить эту гифку?")
            .reply_markup(get_markup())
            .await?;
    }

    dialogue.update(State::Idle).await?;

    Ok(())
}

fn get_markup() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("Press F To Pray", json!(CallbackOperations::AddToFButton).to_string()),
            InlineKeyboardButton::callback("User left", json!(CallbackOperations::AddToUserLeft).to_string()),
        ],
        vec![
            InlineKeyboardButton::callback(
                "Meme already exists",
                json!(CallbackOperations::AddToMemeAlreadyExists).to_string(),
            ),
            InlineKeyboardButton::callback("Newbie User", json!(CallbackOperations::AddToNewbieUser).to_string()),
        ],
        vec![InlineKeyboardButton::callback(
            "Similar Meme",
            json!(CallbackOperations::AddToSimilarMeme).to_string(),
        )],
        vec![InlineKeyboardButton::callback(
            "❌ Cancel",
            json!(CallbackOperations::Cancel).to_string(),
        )],
    ])
}
