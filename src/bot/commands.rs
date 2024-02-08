use std::sync::Arc;

use anyhow::Result;
use teloxide::{prelude::*, utils::command::BotCommands};

use crate::app::Application;
use crate::bot::Bot;

use super::markups::*;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Команды которые поддерживает бот:")]
pub enum PublicCommand {
    #[command(description = "Показывает перечень команд")]
    Help,
    #[command(rename_rule = "UPPERCASE", description = "Press \"F\" to Pray Respects")]
    F,
    #[command(description = "Это даже не баян, это аккордеон на**й")]
    Accordion,
    #[command(description = "Удалить свой мем")]
    UnMeme,
    #[command(description = "Статистика мемочата")]
    Stats,
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Команды которые поддерживает бот:")]
pub enum PrivateCommand {
    #[command(description = "Показывает перечень команд")]
    Help,
    #[command(description = "Отправить сообщение в чат")]
    Message(String),
}

pub struct CommandsHandler {
    pub app: Arc<Application>,
    pub bot: Bot,
    pub msg: Message,
}

impl CommandsHandler {
    pub async fn public_handle(bot: Bot, msg: Message, cmd: PublicCommand, app: Arc<Application>) -> Result<()> {
        let handler = CommandsHandler { app, bot, msg };

        handler.bot.delete_message(handler.msg.chat.id, handler.msg.id).await?;

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
            PublicCommand::Stats => {
                handler.stats_command().await?;
            }
        };

        Ok(())
    }

    pub async fn private_handle(bot: Bot, msg: Message, cmd: PrivateCommand, app: Arc<Application>) -> Result<()> {
        let handler = CommandsHandler { app, bot, msg };

        match cmd {
            PrivateCommand::Help => {
                handler.help_command_private().await?;
            }
            PrivateCommand::Message(text) => {
                handler.message_command(&text).await?;
            }
        };

        Ok(())
    }

    pub async fn help_command_public(&self) -> Result<()> {
        let can_send = self
            .app
            .redis
            .can_send_message("help", self.msg.chat.id.0, self.msg.id.0);

        if !can_send {
            return Ok(());
        }

        self.bot
            .send_message(
                self.msg.chat.id,
                format!(
                    "{}\n\n{}\nВерсия бота: {}",
                    PublicCommand::descriptions(),
                    include_str!("../../messages/help_text_addition.in"),
                    self.app.config.app_version
                ),
            )
            .await?;

        Ok(())
    }

    pub async fn help_command_private(&self) -> Result<()> {
        self.bot
            .send_message(
                self.msg.chat.id,
                format!(
                    "{}\n\nВерсия бота: {}",
                    PrivateCommand::descriptions(),
                    self.app.config.app_version
                ),
            )
            .await?;

        Ok(())
    }

    pub async fn message_command(&self, text: &str) -> Result<()> {
        let user_chats = self
            .app
            .database
            .get_admin_chats(self.msg.from().unwrap().id.0)
            .unwrap_or_default();

        match user_chats.len() {
            0 => {}
            1 => {
                let chat_id = *user_chats.first().unwrap();
                self.bot.send_message(ChatId(chat_id), text).await?;
            }
            2.. => {
                // self.bot
                //     .send_message(self.msg.chat.id, "В какой чат отправить сообщение?")
                //     .reply_markup(InlineKeyboardMarkup::new(user_chats.iter().map(|chat| {
                //         vec![InlineKeyboardButton::callback(
                //             chat.to_string(),
                //             format!("data: {}", chat),
                //         )]
                //     })))
                //     .await?;
            }
        }

        Ok(())
    }

    pub async fn f_command(&self) -> Result<()> {
        self.bot.send_message(self.msg.chat.id, String::from("F")).await?;

        Ok(())
    }

    pub async fn accordion_command(&self) -> Result<()> {
        let me = self.bot.get_me().await?;

        match self.msg.reply_to_message() {
            Some(repl) => {
                if repl.from().unwrap().id != me.id {
                    return Ok(());
                }

                let can_send = self
                    .app
                    .redis
                    .can_send_message("accordion", self.msg.chat.id.0, self.msg.id.0);

                if !can_send {
                    return Ok(());
                }

                let meme = self.app.database.get_meme_by_msg_id(repl.id.0 as i64, repl.chat.id.0)?;
                let user_res = self.bot.get_chat_member(self.msg.chat.id, meme.user_id()).await;
                let mut user_text = String::new();

                if user_res.is_ok() {
                    user_text = format!("{}!\n", crate::app::utils::get_user_text(&user_res.unwrap().user));
                }

                self.bot
                    .send_message(
                        self.msg.chat.id,
                        format!("{user_text} Пользователи жалуются на великое баянище!\nЧто будем с ним делать?"),
                    )
                    .reply_to_message_id(repl.id)
                    .reply_markup(
                        DeleteMarkup::new(meme.uuid)
                            .set_ok_text("👎 Удалите, прошу прощения")
                            .set_none_text("👍 Беру на себя ответственность")
                            .get_markup(),
                    )
                    .await?;
            }
            None => {
                let can_send = self
                    .app
                    .redis
                    .can_send_message("accordion_none", self.msg.chat.id.0, self.msg.id.0);

                if can_send {
                    self.bot
                        .send_message(
                            self.msg.chat.id,
                            String::from("Чтобы пожаловаться на сообщение, на него нужно ответить!"),
                        )
                        .await?;
                }
            }
        }

        Ok(())
    }

    pub async fn unmeme_command(&self) -> Result<()> {
        let me = self.bot.get_me().await?;

        match self.msg.reply_to_message() {
            Some(repl) => {
                if repl.from().unwrap().id != me.id {
                    return Ok(());
                }

                let can_send = self
                    .app
                    .redis
                    .can_send_message("unmeme", self.msg.chat.id.0, self.msg.id.0);

                if !can_send {
                    return Ok(());
                }

                let meme = self
                    .app
                    .database
                    .get_meme_by_msg_id(repl.id.0 as i64, repl.chat.id.0)
                    .unwrap();

                self.bot
                    .send_message(self.msg.chat.id, String::from("Ты хочешь удалить мем?"))
                    .reply_to_message_id(repl.id)
                    .reply_markup(
                        DeleteMarkup::new(meme.uuid)
                            .set_ok_text("🗑 Да, я хочу удалить")
                            .set_none_text("❌ Нет, я передумал(а)")
                            .get_markup(),
                    )
                    .await?;
            }
            None => {
                let can_send = self
                    .app
                    .redis
                    .can_send_message("unmeme_none", self.msg.chat.id.0, self.msg.id.0);
                if can_send {
                    self.bot
                        .send_message(
                            self.msg.chat.id,
                            String::from("Чтобы удалить свой мем, нужно ответить на него!"),
                        )
                        .await?;
                }
            }
        }

        Ok(())
    }

    pub async fn stats_command(&self) -> Result<()> {
        let can_send = self
            .app
            .redis
            .can_send_message("stats", self.msg.chat.id.0, self.msg.id.0);

        if !can_send {
            return Ok(());
        }

        let memes_count = self.app.database.get_memes_count(self.msg.chat.id.0);
        let likes_count = self.app.database.get_meme_likes_count(self.msg.chat.id.0);
        let dislikes_count = self.app.database.get_meme_dislikes_count(self.msg.chat.id.0);

        let message = include_str!("../../messages/stats.in")
            .replace("{memes_count}", &memes_count.to_string())
            .replace("{memes_likes}", &likes_count.to_string())
            .replace("{memes_dislikes}", &dislikes_count.to_string());

        self.bot.send_message(self.msg.chat.id, message).await?;

        Ok(())
    }
}
