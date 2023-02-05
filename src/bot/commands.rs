use std::sync::Arc;

use anyhow::Result;
use teloxide::{prelude::*, utils::command::BotCommands};

use crate::database::models::AddChat;
use crate::Application;
use crate::bot::Bot;

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
    #[command(description = "Зарегистрировать чат (только для админов)")]
    Register,
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "Команды которые поддерживает бот:"
)]
pub enum PrivateCommand {
    #[command(description = "Показывает перечень команд")]
    Help,
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
    ) -> Result<()> {
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
            PublicCommand::Register => {
                handler.register_command().await?;
            }
        };

        Ok(())
    }

    pub async fn private_handle(
        bot: Bot,
        msg: Message,
        cmd: PrivateCommand,
        app: Arc<Application>,
    ) -> Result<()> {
        let handler = CommandsHandler { app, bot, msg };

        match cmd {
            PrivateCommand::Help => {
                handler.help_command_private().await?;
            }
        };

        Ok(())
    }

    pub async fn help_command_public(&self) -> Result<()> {
        self.bot
            .send_message(
                self.msg.chat.id,
                format!(
                    "{}\n\n{}\nВерсия бота: {}",
                    PublicCommand::descriptions(),
                    include_str!("../../messages/help_text_addition.in"),
                    self.app.version
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
                    self.app.version
                ),
            )
            .await?;

        Ok(())
    }

    pub async fn register_command(&self) -> Result<()> {
        self.bot
            .delete_message(self.msg.chat.id, self.msg.id)
            .await?;

        if self.app.redis.is_chat_registered(self.msg.chat.id.0) {
            return Ok(());
        }

        let admins = self.bot.get_chat_administrators(self.msg.chat.id).await?;
        let uids = admins.iter().map(|m| m.user.id.0).collect::<Vec<u64>>();

        if !uids.contains(&self.msg.from().unwrap().id.0) {
            return Ok(());
        }

        self.app.redis.register_chat(self.msg.chat.id.0);
        let _ = self
            .app
            .database
            .add_chat(&AddChat::new_from_tg(&self.msg.chat));

        self.bot
            .send_message(self.msg.chat.id, "Чат успешно зарегистрирован!")
            .await?;

        Ok(())
    }

    pub async fn f_command(&self) -> Result<()> {
        self.bot
            .send_message(self.msg.chat.id, String::from("F"))
            .await?;

        Ok(())
    }

    pub async fn accordion_command(&self) -> Result<()> {
        match self.msg.reply_to_message() {
            Some(repl) => {
                if repl.from().unwrap().id == self.app.bot.id {
                    let meme = self
                        .app
                        .database
                        .get_meme_by_msg_id(repl.id.0 as i64, repl.chat.id.0)?;
                    let user_res = self
                        .bot
                        .get_chat_member(self.msg.chat.id, meme.user_id())
                        .await;
                    let mut user_text = String::new();

                    if user_res.is_ok() {
                        user_text = format!(
                            "{}!\n",
                            crate::utils::get_user_text(&user_res.unwrap().user)
                        );
                    }

                    self.bot
                        .delete_message(self.msg.chat.id, self.msg.id)
                        .await?;
                    self.bot
                        .send_message(
                            self.msg.chat.id,
                            format!("{user_text} Пользователи жалуются на великое баянище!\nЧто будем с ним делать?")
                        )
                        .reply_to_message_id(repl.id)
                        .reply_markup(
                            DeleteMarkup::new(meme.uuid)
                                .set_ok_text(&format!(
                                    "{} Удалите, прошу прощения",
                                    emojis::get_by_shortcode("thumbsdown").unwrap().as_str()
                                ))
                                .set_none_text(&format!(
                                    "{} Беру на себя ответственность",
                                    emojis::get_by_shortcode("thumbsup").unwrap().as_str()
                                ))
                                .get_markup()
                        )
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
                        String::from("Чтобы пожаловаться на сообщение, на него нужно ответить!"),
                    )
                    .await?;
            }
        }

        Ok(())
    }

    pub async fn unmeme_command(&self) -> Result<()> {
        match self.msg.reply_to_message() {
            Some(repl) => {
                if repl.from().unwrap().id == self.app.bot.id {
                    let meme = self
                        .app
                        .database
                        .get_meme_by_msg_id(repl.id.0 as i64, repl.chat.id.0)
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
                        .reply_markup(
                            DeleteMarkup::new(meme.uuid)
                                .set_ok_text(&format!(
                                    "{} Да, я хочу удалить",
                                    emojis::get_by_shortcode("wastebasket").unwrap().as_str()
                                ))
                                .set_none_text(&format!(
                                    "{} Нет, я передумал(а)",
                                    emojis::get_by_shortcode("x").unwrap().as_str()
                                ))
                                .get_markup(),
                        )
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
