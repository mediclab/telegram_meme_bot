use std::error::Error;
use std::sync::Arc;

use crate::bot::markups::*;
use crate::database::models::Meme;
use crate::Application;
use teloxide::prelude::*;

pub struct CallbackHandler {
    pub app: Arc<Application>,
    pub bot: Bot,
    pub callback: CallbackQuery,
}

impl CallbackHandler {
    pub async fn handle(
        bot: Bot,
        callback: CallbackQuery,
        app: Arc<Application>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let handler = CallbackHandler { app, bot, callback };

        let data: MemeCallback = serde_json::from_str(
            &handler
                .callback
                .data
                .clone()
                .unwrap_or_else(|| r#"{}"#.to_string()),
        )?;

        let meme = handler.app.database.get_meme(&data.uuid)?;

        match data.op {
            CallbackOperations::Like => {
                handler.like(&meme).await?;
            }
            CallbackOperations::Dislike => {
                handler.dislike(&meme).await?;
            }
            CallbackOperations::Delete => {
                handler.delete(&meme).await?;
            }
            CallbackOperations::None => {
                handler.none(&meme).await?;
            }
        };

        Ok(())
    }

    pub async fn like(&self, meme: &Meme) -> Result<(), Box<dyn Error + Send + Sync>> {
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

    pub async fn dislike(&self, meme: &Meme) -> Result<(), Box<dyn Error + Send + Sync>> {
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

    pub async fn none(&self, meme: &Meme) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let msg = self.callback.message.as_ref().unwrap();

        if meme.user_id != self.callback.from.id.0 as i64 {
            self.bot
                .answer_callback_query(&self.callback.id)
                .text("Только пользователь отправивший мем, может сделать это")
                .show_alert(true)
                .await?;

            return Ok(false);
        }

        self.bot.delete_message(msg.chat.id, msg.id).await?;

        Ok(true)
    }

    pub async fn delete(&self, meme: &Meme) -> Result<(), Box<dyn Error + Send + Sync>> {
        if !self.none(meme).await? {
            return Ok(());
        }

        self.bot
            .delete_message(meme.chat_id(), meme.msg_id())
            .await?;

        self.app.database.delete_meme(&meme.uuid);

        Ok(())
    }

    async fn update_message(
        &self,
        meme: &Meme,
        msg: &Message,
        counts: (i64, i64),
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let (likes, dislikes) = counts;
        let meme_markup = MemeMarkup::new(likes, dislikes, meme.uuid);

        self.bot
            .edit_message_reply_markup(msg.chat.id, msg.id)
            .reply_markup(meme_markup.get_markup())
            .await?;

        Ok(())
    }
}
