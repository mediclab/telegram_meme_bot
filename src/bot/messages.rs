use std::error::Error;
use std::sync::Arc;
use teloxide::{
    prelude::*,
    types::{InputFile, MessageKind, ReplyMarkup},
};

use crate::bot::markups::*;
use crate::bot::utils as Utils;
use crate::database::models::User;
use crate::database::repository::{MemeRepository, UserRepository};
use crate::Application;

pub struct MessagesHandler {
    pub app: Arc<Application>,
    pub bot: Bot,
    pub msg: Message,
}

impl MessagesHandler {
    pub async fn handle(
        bot: Bot,
        msg: Message,
        app: Arc<Application>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let handler = MessagesHandler { app, bot, msg };

        match &handler.msg.kind {
            MessageKind::Common(_) => {
                handler.common().await?;
            }
            MessageKind::NewChatMembers(_) => {
                handler.newbie().await?;
            }
            MessageKind::LeftChatMember(_) => {
                handler.left().await?;
            }
            _ => {}
        };

        Ok(())
    }

    pub async fn common(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        // If This is forwarded message - nothing to do.
        if self.msg.forward().is_some() {
            return Ok(());
        }

        // If caption contains "nomeme" - nothing to do.
        if self.msg.caption().unwrap_or("").contains("nomeme") {
            return Ok(());
        }

        if self.msg.from().is_none() {
            return Err(Box::try_from("User is anonymous!").unwrap());
        }

        let user = self.msg.from().unwrap();
        let meme_repository = MemeRepository::new(self.app.database.clone());
        let user_text = Utils::get_user_text(user);

        if let Some(photos) = self.msg.photo() {
            let meme = meme_repository
                .add(
                    self.msg.from().unwrap().id.0 as i64,
                    self.msg.chat.id.0,
                    serde_json::json!(self.msg.photo()),
                )
                .unwrap();

            self.bot
                .delete_message(self.msg.chat.id, self.msg.id)
                .await?;

            let markup = MemeMarkup::new(0, 0, meme.uuid);
            let bot_msg = self
                .bot
                .send_photo(self.msg.chat.id, InputFile::file_id(&photos[0].file.id))
                .caption(format!("Оцените мем {}", user_text))
                .reply_markup(ReplyMarkup::InlineKeyboard(markup.get_markup()))
                .await?;

            meme_repository.add_msg_id(&meme.uuid, bot_msg.id.0 as i64);
        }

        if let Some(video) = self.msg.video() {
            let meme = meme_repository
                .add(
                    self.msg.from().unwrap().id.0 as i64,
                    self.msg.chat.id.0,
                    serde_json::json!(self.msg.video()),
                )
                .unwrap();

            self.bot
                .delete_message(self.msg.chat.id, self.msg.id)
                .await?;

            let markup = MemeMarkup::new(0, 0, meme.uuid);
            let bot_msg = self
                .bot
                .send_video(self.msg.chat.id, InputFile::file_id(&video.file.id))
                .caption(format!("Оцените видео-мем {}", user_text))
                .reply_markup(ReplyMarkup::InlineKeyboard(markup.get_markup()))
                .await?;

            meme_repository.add_msg_id(&meme.uuid, bot_msg.id.0 as i64);
        }

        Ok(())
    }

    pub async fn newbie(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let repository = UserRepository::new(self.app.database.clone());
        let messages = Utils::Messages::load(include_str!("../../messages/newbie.in"));

        self.bot
            .delete_message(self.msg.chat.id, self.msg.id)
            .await?;

        let users = self
            .msg
            .new_chat_members()
            .expect("New chat members not found!");

        let users_names = users
            .iter()
            .map(Utils::get_user_text)
            .collect::<Vec<String>>()
            .join(", ");

        self.bot
            .send_message(
                self.msg.chat.id,
                messages.random().replace("{user_name}", &users_names),
            )
            .await?;

        users.iter().for_each(|user| {
            let _ = repository.add(&User::new_from_tg(user));
        });

        Ok(())
    }

    pub async fn left(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let repository = UserRepository::new(self.app.database.clone());
        let messages = Utils::Messages::load(include_str!("../../messages/left.in"));

        self.bot
            .delete_message(self.msg.chat.id, self.msg.id)
            .await?;

        let user = self.msg.left_chat_member().expect("Left users not found!");

        self.bot
            .send_message(
                self.msg.chat.id,
                messages
                    .random()
                    .replace("{user_name}", &Utils::get_user_text(user)),
            )
            .await?;

        repository.delete(user.id.0 as i64);

        Ok(())
    }
}
