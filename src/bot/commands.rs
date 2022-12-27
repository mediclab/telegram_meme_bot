use teloxide::{prelude::*, utils::command::BotCommands};
use std::sync::Arc;
use std::error::Error;

use crate::BotState;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "These commands are supported:")]
pub enum Command {
    #[command(description = "display this text.")]
    Help,
}

pub async fn handle(bot: Bot, msg: Message, cmd: Command, _state: Arc<BotState>) -> Result<(), Box<dyn Error + Send + Sync>> {
    match cmd {
        Command::Help => bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?,
    };

    Ok(())
}