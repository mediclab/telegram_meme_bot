use std::sync::Arc;

use anyhow::Result;
use teloxide::dispatching::dialogue::GetChatId;
use teloxide::prelude::*;

use crate::app::Application;
use crate::bot::markups::*;
use crate::bot::Bot;
use crate::database::models::Meme;

pub struct CallbackHandler {
    pub app: Arc<Application>,
    pub bot: Bot,
    pub callback: CallbackQuery,
}

impl CallbackHandler {
    pub async fn handle(bot: Bot, callback: CallbackQuery, app: Arc<Application>) -> Result<()> {
        let handler = CallbackHandler { app, bot, callback };

        match &handler.callback.chat_id() {
            Some(chat) => {
                if chat.0 > 0 {
                    handler.private_handle().await?;
                } else {
                    handler.public_handle().await?;
                }
            }

            None => {}
        }

        Ok(())
    }

    pub async fn private_handle(&self) -> Result<()> {
        Ok(())
    }

    pub async fn public_handle(&self) -> Result<()> {
        let data: MemeCallback = serde_json::from_str(
            &self
                .callback
                .data
                .clone()
                .unwrap_or_else(|| r#"{}"#.to_string()),
        )?;

        let meme = self.app.database.get_meme(&data.uuid)?;

        match data.op {
            CallbackOperations::Like => {
                self.like(&meme).await?;
            }
            CallbackOperations::Dislike => {
                self.dislike(&meme).await?;
            }
            CallbackOperations::Delete => {
                self.delete(&meme).await?;
            }
            CallbackOperations::None => {
                self.none(&meme).await?;
            }
        };

        Ok(())
    }

    pub async fn like(&self, meme: &Meme) -> Result<()> {
        let msg = self.callback.message.clone().unwrap();
        let user_id = self.callback.from.id.0 as i64;
        let repository = &self.app.database;

        if repository.like_exists(user_id, &meme.uuid) {
            repository.cancel_like(user_id, &meme.uuid);
        } else {
            repository.like(user_id, &meme.uuid);
        }

        let likes = (
            repository.count_likes(&meme.uuid),
            repository.count_dislikes(&meme.uuid),
        );

        self.update_message(meme, &msg, likes).await?;

        Ok(())
    }

    pub async fn dislike(&self, meme: &Meme) -> Result<()> {
        let msg = self.callback.message.clone().unwrap();
        let user_id = self.callback.from.id.0 as i64;
        let repository = &self.app.database;

        if repository.dislike_exists(user_id, &meme.uuid) {
            repository.cancel_dislike(user_id, &meme.uuid);
        } else {
            repository.dislike(user_id, &meme.uuid);
        }

        let likes = (
            repository.count_likes(&meme.uuid),
            repository.count_dislikes(&meme.uuid),
        );

        self.update_message(meme, &msg, likes).await?;

        Ok(())
    }

    pub async fn none(&self, meme: &Meme) -> Result<()> {
        let msg = self.callback.message.as_ref().unwrap();

        if !self.can_user_interact(meme) {
            self.bot
                .answer_callback_query(&self.callback.id)
                .text("Только тот, кто прислал мем (или админ), может сделать это")
                .show_alert(true)
                .await?;

            return Ok(());
        }

        self.bot.delete_message(msg.chat.id, msg.id).await?;

        self.bot
            .answer_callback_query(&self.callback.id)
            .text("Штош, на Вашей совести")
            .await?;

        Ok(())
    }

    pub async fn delete(&self, meme: &Meme) -> Result<()> {
        let msg = self.callback.message.as_ref().unwrap();

        if !self.can_user_interact(meme) {
            self.bot
                .answer_callback_query(&self.callback.id)
                .text("Только тот, кто прислал мем (или админ), может сделать это")
                .show_alert(true)
                .await?;

            return Ok(());
        }

        self.bot.delete_message(msg.chat.id, msg.id).await?;

        self.bot
            .delete_message(meme.chat_id(), meme.msg_id())
            .await?;

        self.app.database.delete_meme(&meme.uuid);

        self.bot
            .answer_callback_query(&self.callback.id)
            .text("УдОлено")
            .await?;

        Ok(())
    }

    async fn update_message(&self, meme: &Meme, msg: &Message, counts: (i64, i64)) -> Result<()> {
        let (likes, dislikes) = counts;
        let meme_markup = MemeMarkup::new(likes, dislikes, meme.uuid);

        self.bot
            .edit_message_reply_markup(msg.chat.id, msg.id)
            .reply_markup(meme_markup.get_markup())
            .await?;

        self.bot
            .answer_callback_query(&self.callback.id)
            .text("Круто, что тебе не пофиг")
            .await?;

        Ok(())
    }

    fn can_user_interact(&self, meme: &Meme) -> bool {
        let admins = self
            .app
            .redis
            .get_chat_admins(self.callback.chat_id().unwrap().0);
        let is_user_admin = admins.contains(&self.callback.from.id.0);

        is_user_admin || (meme.user_id == self.callback.from.id.0 as i64)
    }
}
