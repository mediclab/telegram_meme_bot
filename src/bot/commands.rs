use std::error::Error;
use std::sync::Arc;
use teloxide::{prelude::*, utils::command::BotCommands};

use crate::database::repository::MemeRepository;
use crate::Application;

use super::markups::*;

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "Команды которые поддерживает бот:"
)]
pub enum Command {
    #[command(description = "Показывает перечень команд")]
    Help,
    #[command(
        rename_rule = "UPPERCASE",
        description = "Press \"F\" to Pray Respects"
    )]
    F,
    #[command(description = "Это даже не баян, это аккордеон на**й")]
    Accordeon,
    #[command(description = "Удалить свой мем")]
    UnMeme,
}

pub struct CommandsHandler {
    pub app: Arc<Application>,
    pub bot: Bot,
    pub msg: Message,
    pub cmd: Command,
}

impl CommandsHandler {
    pub async fn handle(
        bot: Bot,
        msg: Message,
        cmd: Command,
        app: Arc<Application>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let handler = CommandsHandler { app, bot, msg, cmd };

        if handler.msg.chat.id.0 > 0 {
            return handler.private().await;
        }

        match handler.cmd {
            Command::Help => {
                handler.help_command().await?;
            }
            Command::F => {
                handler.f_command().await?;
            }
            Command::Accordeon => {
                handler.accordeon_command().await?;
            }
            Command::UnMeme => {
                handler.unmeme_command().await?;
            }
        };

        Ok(())
    }

    pub async fn private(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.bot
            .send_message(
                self.msg.chat.id,
                String::from("Временно недоступно в приватных чатах"),
            )
            .await?;

        Ok(())
    }

    pub async fn help_command(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.bot
            .send_message(self.msg.chat.id, Command::descriptions().to_string())
            .await?;

        Ok(())
    }

    pub async fn f_command(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.bot
            .send_message(self.msg.chat.id, String::from("F"))
            .await?;

        Ok(())
    }

    pub async fn accordeon_command(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let repository = MemeRepository::new(self.app.database.clone());

        match self.msg.reply_to_message() {
            Some(repl) => {
                if repl.from().unwrap().id == self.app.bot.id {
                    let meme = repository.get_by_msg_id(repl.id.0 as i64, repl.chat.id.0)?;

                    self.bot
                        .delete_message(self.msg.chat.id, self.msg.id)
                        .await?;
                    self.bot
                        .send_message(
                            self.msg.chat.id,
                            String::from("Пользователи жалуются на великое баянище!\nЧто будем с ним делать?")
                        )
                        .reply_to_message_id(repl.id)
                        .reply_markup(
                            AccordeonMarkup::new(meme.uuid).get_markup()
                        ).await?;
                } else {
                    Err("Reply message is not from bot")?
                }
            }
            None => {
                self.bot
                    .delete_message(self.msg.chat.id, self.msg.id)
                    .await?;

                self.bot
                    .send_message(
                        self.msg.chat.id,
                        String::from("Чтобы пожаловаться на сообщение, на него нужно ответить!"),
                    )
                    .await?;
            }
        }

        Ok(())
    }

    pub async fn unmeme_command(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let repository = MemeRepository::new(self.app.database.clone());

        match self.msg.reply_to_message() {
            Some(repl) => {
                if repl.from().unwrap().id == self.app.bot.id {
                    let meme = repository
                        .get_by_msg_id(repl.id.0 as i64, repl.chat.id.0)
                        .unwrap();

                    self.bot
                        .delete_message(self.msg.chat.id, self.msg.id)
                        .await?;
                    self.bot
                        .send_message(
                            self.msg.chat.id,
                            String::from("Вы действительно хотите удалить мем?"),
                        )
                        .reply_to_message_id(repl.id)
                        .reply_markup(DeleteMarkup::new(meme.uuid).get_markup())
                        .await?;
                } else {
                    Err("Reply message is not from bot")?
                }
            }
            None => {
                self.bot
                    .delete_message(self.msg.chat.id, self.msg.id)
                    .await?;

                self.bot
                    .send_message(
                        self.msg.chat.id,
                        String::from("Чтобы удалить свой мем, нужно ответить на него!"),
                    )
                    .await?;
            }
        }

        Ok(())
    }
}
