use super::types::CallbackOperations;
use crate::bot::{Bot, BotDialogue, State};
use crate::database::entity::{messages::MessageTypes, prelude::Messages};
use anyhow::Result;
use teloxide::{payloads::AnswerCallbackQuerySetters, prelude::*};

pub async fn handle(bot: Bot, callback: CallbackQuery, dialogue: BotDialogue) -> Result<()> {
    let data: CallbackOperations = serde_json::from_str(&callback.data.clone().unwrap_or_else(|| r#"{}"#.to_string()))?;

    let msg = if let Some(m) = callback.message {
        m
    } else {
        return Ok(());
    };

    if let CallbackOperations::Cancel = data {
        bot.answer_callback_query(callback.id)
            .text("Галя, у нас отмена!")
            .await?;
        dialogue.update(State::Idle).await?;

        bot.delete_message(msg.chat.id, msg.id).await?;
        return Ok(());
    };

    let (msg_type, text) = if let Some(t) = msg.text() {
        (MessageTypes::Text, t.to_string())
    } else if let Some(p) = msg.photo() {
        (MessageTypes::Photo, p[0].file.id.clone())
    } else if let Some(d) = msg.document() {
        (MessageTypes::Document, d.file.id.clone())
    } else {
        return Ok(());
    };

    Messages::add(msg_type, data.into(), &text).await;

    bot.answer_callback_query(callback.id).text("Добавил!").await?;
    bot.delete_message(msg.chat.id, msg.id).await?;
    dialogue.update(State::Idle).await?;

    Ok(())
}
