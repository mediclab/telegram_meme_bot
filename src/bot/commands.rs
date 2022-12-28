use teloxide::{prelude::*, utils::command::BotCommands};
use std::sync::Arc;
use std::error::Error;

use crate::BotState;
use crate::database::repository::MemeRepository;

use super::markups::AccordeonMarkup;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Команды которые поддерживает бот:")]
pub enum Command {
    #[command(description = "Показывает перечень команд")]
    Help,
    #[command(rename_rule = "UPPERCASE", description = "Press \"F\" to Pray Respects")]
    F,
    #[command(description = "Это даже не баян, это аккордеон на**й")]
    Accordeon,
    #[command(description = "Удалить свой мем")]
    UnMeme,
}

pub async fn handle(bot: Bot, msg: Message, cmd: Command, state: Arc<BotState>) -> Result<(), Box<dyn Error + Send + Sync>> {
    match cmd {
        Command::Help => bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?,
        Command::F => bot.send_message(msg.chat.id, String::from("F")).await?,
        Command::Accordeon => {
            match msg.reply_to_message() {
                Some(repl) => {
                    if repl.from().unwrap().id == state.bot.id {
                        let repository = MemeRepository::new(state.db_manager.clone());
                        let meme = repository.get_by_msg_id(repl.id.0 as i64).unwrap();
                        let accordeon_markup = AccordeonMarkup::new(meme.uuid);

                        bot.delete_message(msg.chat.id, msg.id).await?;
                        bot
                            .send_message(msg.chat.id, String::from("Пользователи жалуются на великое баянище!\nЧто будем с ним делать?"))
                            .reply_to_message_id(repl.id)
                            .reply_markup(accordeon_markup.get_markup())
                        .await?
                    } else {
                        Err("Reply message is not from bot")?
                    }
                },
                None => bot.send_message(msg.chat.id, String::from("Чтобы пожаловаться на сообщение, на него нужно ответить!")).await?,
            }
        },
        Command::UnMeme => {
            match msg.reply_to_message() {
                Some(_repl) => {
                    bot.send_message(msg.chat.id, String::from("Удалить мемес")).await?
                },
                None => bot.send_message(msg.chat.id, String::from("Чтобы удалить свой мем, нужно ответить на него!")).await?
            }
        },
    };

    Ok(())
}