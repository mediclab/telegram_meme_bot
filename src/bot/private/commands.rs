use crate::app::Application;
use crate::bot::Bot;
use std::sync::Arc;
use teloxide::{prelude::*, utils::command::BotCommands};

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Команды которые поддерживает бот:")]
pub enum PrivateCommand {
    #[command(description = "Показывает перечень команд")]
    Help,
}

pub async fn help_command(bot: Bot, msg: Message, app: Arc<Application>) -> anyhow::Result<()> {
    bot.send_message(
        msg.chat.id,
        format!(
            "{}\n\nВерсия бота: {}",
            PrivateCommand::descriptions(),
            app.config.app_version
        ),
    )
    .await?;

    Ok(())
}
