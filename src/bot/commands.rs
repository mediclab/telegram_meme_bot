use teloxide::{prelude::*, utils::command::BotCommands};
use std::sync::Arc;
use std::error::Error;

use crate::Application;
use crate::database::repository::MemeRepository;

use super::markups::*;

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

pub async fn handle(bot: Bot, msg: Message, cmd: Command, app: Arc<Application>) -> Result<(), Box<dyn Error + Send + Sync>> {
    if msg.chat.id.0 > 0 {
        // bot.send_message(msg.chat.id, String::from("Временно недоступно в приватных чатах")).await?;
        // Err("Temporary disabled in private chats")?
    }

    let repository = MemeRepository::new(app.database.clone());

    match cmd {
        Command::Help => bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?,
        Command::F => bot.send_message(msg.chat.id, String::from("F")).await?,
        Command::Accordeon => {
            match msg.reply_to_message() {
                Some(repl) => {
                    if repl.from().unwrap().id == app.bot.id {
                        let meme = repository.get_by_msg_id(repl.id.0 as i64, repl.chat.id.0).unwrap();
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
                }
                None => bot.send_message(msg.chat.id, String::from("Чтобы пожаловаться на сообщение, на него нужно ответить!")).await?,
            }
        }
        Command::UnMeme => {
            match msg.reply_to_message() {
                Some(repl) => {
                    if repl.from().unwrap().id == app.bot.id {
                        let meme = repository.get_by_msg_id(repl.id.0 as i64, repl.chat.id.0).unwrap();
                        let delete_markup = DeleteMarkup::new(meme.uuid);

                        bot.delete_message(msg.chat.id, msg.id).await?;
                        bot.send_message(msg.chat.id, String::from("Вы действительно хотите удалить мем?"))
                            .reply_to_message_id(repl.id)
                            .reply_markup(delete_markup.get_markup())
                            .await?
                    } else {
                        Err("Reply message is not from bot")?
                    }
                }
                None => bot.send_message(msg.chat.id, String::from("Чтобы удалить свой мем, нужно ответить на него!")).await?
            }
        }
    };

    Ok(())
}