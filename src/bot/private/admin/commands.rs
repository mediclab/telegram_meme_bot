use crate::app::Application;
use crate::bot::{private::PrivateState, Bot, BotDialogue, State};
use crate::database::entity::prelude::ChatAdmins;
use std::sync::Arc;
use teloxide::{prelude::*, utils::command::BotCommands};

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Команды которые поддерживает бот:")]
pub enum AdminCommand {
    #[command(description = "Показывает перечень команд")]
    Help,
    #[command(description = "Отправить сообщение в чат")]
    Message(String),
    #[command(description = "Добавить мем в базу")]
    AddMessage,
}

pub async fn help_command(bot: Bot, msg: Message, app: Arc<Application>) -> anyhow::Result<()> {
    bot.send_message(
        msg.chat.id,
        format!(
            "{}\n\nВерсия бота: {}",
            AdminCommand::descriptions(),
            app.config.app_version
        ),
    )
    .await?;

    Ok(())
}

pub async fn message_command(bot: Bot, msg: Message, text: String) -> anyhow::Result<()> {
    let user_chats = ChatAdmins::get_admin_chats(msg.from.unwrap().id.0).await;

    match user_chats.len() {
        0 => {
            warn!("User is not admin");
        }
        1 => {
            let chat_id = *user_chats.first().unwrap();
            bot.send_message(ChatId(chat_id), text).await?;
        }
        2.. => {
            warn!("User have 2 or many chats");
        }
    }

    Ok(())
}

pub async fn add_message_command(bot: Bot, msg: Message, dialogue: BotDialogue) -> anyhow::Result<()> {
    bot.send_message(
        msg.chat.id,
        "Отправьте изображение, текст или гифку для добавления ее в хранилище",
    )
    .await?;

    dialogue.update(State::Private(PrivateState::AdminAddMessage)).await?;
    Ok(())
}
