use std::error::Error;
use std::sync::Arc;
use teloxide::types::ParseMode;
use teloxide::{prelude::*, utils::command::BotCommands};

use crate::database::models::Chat;
use crate::database::repository::{ChatRepository, MemeRepository};
use crate::Application;

use super::markups::*;

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "Команды которые поддерживает бот:"
)]
pub enum PublicCommand {
    #[command(description = "Показывает перечень команд")]
    Help,
    #[command(
        rename_rule = "UPPERCASE",
        description = "Press \"F\" to Pray Respects"
    )]
    F,
    #[command(description = "Это даже не баян, это аккордеон на**й")]
    Accordion,
    #[command(description = "Удалить свой мем")]
    UnMeme,
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "Команды которые поддерживает бот:"
)]
pub enum PrivateCommand {
    #[command(description = "Показывает перечень команд")]
    Help,
    #[command(description = "Зарегистрировать чат")]
    Register(String),
}

pub struct CommandsHandler {
    pub app: Arc<Application>,
    pub bot: Bot,
    pub msg: Message,
}

impl CommandsHandler {
    pub async fn public_handle(
        bot: Bot,
        msg: Message,
        cmd: PublicCommand,
        app: Arc<Application>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let handler = CommandsHandler { app, bot, msg };

        match cmd {
            PublicCommand::Help => {
                handler.help_command_public().await?;
            }
            PublicCommand::F => {
                handler.f_command().await?;
            }
            PublicCommand::Accordion => {
                handler.accordion_command().await?;
            }
            PublicCommand::UnMeme => {
                handler.unmeme_command().await?;
            }
        };

        Ok(())
    }

    pub async fn private_handle(
        bot: Bot,
        msg: Message,
        cmd: PrivateCommand,
        app: Arc<Application>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let handler = CommandsHandler { app, bot, msg };

        match cmd {
            PrivateCommand::Help => {
                handler.help_command_private().await?;
            }
            PrivateCommand::Register(chat_id) => {
                handler
                    .register_command(chat_id.trim().parse::<i64>().unwrap())
                    .await?;
            }
        };

        Ok(())
    }

    pub async fn help_command_public(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.bot
            .send_message(
                self.msg.chat.id,
                format!(
                    "{}\n\nВерсия бота: {}\nIssue\\Предложения: <a href=\"https://github.com/mediclab/telegram_meme_bot/issues\">сюды</a>",
                    PublicCommand::descriptions(),
                    self.app.version
                ),
            )
            .parse_mode(ParseMode::Html)
            .await?;

        Ok(())
    }

    pub async fn help_command_private(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.bot
            .send_message(
                self.msg.chat.id,
                format!(
                    "{}\n\nВерсия бота: {}",
                    PrivateCommand::descriptions(),
                    self.app.version
                ),
            )
            .parse_mode(ParseMode::Html)
            .await?;

        Ok(())
    }

    pub async fn register_command(&self, chat_id: i64) -> Result<(), Box<dyn Error + Send + Sync>> {
        let repository = ChatRepository::new(self.app.database.clone());

        let chat = match self.bot.get_chat(ChatId(chat_id)).await {
            Ok(c) => c,
            Err(_) => {
                self.bot
                    .send_message(
                        self.msg.chat.id,
                        "Не удалось зарегистрировать чат.\nНеверный id чата. Либо чат приватный.",
                    )
                    .await?;

                return Ok(());
            }
        };

        let _ = repository.add(&Chat {
            chat_id: chat.id.0,
            chatname: chat
                .username()
                .expect("Chat name does not exists")
                .to_string(),
            description: chat.description().map(|d| d.to_string()),
            created_at: None,
        });

        self.bot
            .send_message(self.msg.chat.id, "Чат успешно зарегистрирован!")
            .await?;

        Ok(())
    }

    pub async fn f_command(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.bot
            .send_message(self.msg.chat.id, String::from("F"))
            .await?;

        Ok(())
    }

    pub async fn accordion_command(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let repository = MemeRepository::new(self.app.database.clone());

        match self.msg.reply_to_message() {
            Some(repl) => {
                if repl.from().unwrap().id == self.app.bot.id {
                    let meme = repository.get_by_msg_id(repl.id.0 as i64, repl.chat.id.0)?;
                    let user_res = self
                        .bot
                        .get_chat_member(self.msg.chat.id, meme.user_id())
                        .await;
                    let mut user_text = String::new();

                    if user_res.is_ok() {
                        user_text = format!(
                            "{}!\n",
                            crate::bot::utils::get_user_text(&user_res.unwrap().user)
                        );
                    }

                    self.bot
                        .delete_message(self.msg.chat.id, self.msg.id)
                        .await?;
                    self.bot
                        .send_message(
                            self.msg.chat.id,
                            format!("{}Пользователи жалуются на великое баянище!\nЧто будем с ним делать?", user_text)
                        )
                        .reply_to_message_id(repl.id)
                        .reply_markup(
                            AccordionMarkup::new(meme.uuid).get_markup()
                        ).await?;
                } else {
                    return Ok(());
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
                    return Ok(());
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
