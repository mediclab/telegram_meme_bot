use std::sync::Arc;

use anyhow::Result;
use teloxide::dispatching::dialogue::GetChatId;
use teloxide::prelude::*;

use super::markups::*;
use super::types::*;
use crate::app::Application;
use crate::bot::Bot;
use crate::database::entity::{meme_likes::MemeLikesCountAll, memes::Model as MemeModel, prelude::Memes};
use crate::redis::RedisManager;

pub struct CallbackHandler {
    #[allow(dead_code)]
    pub app: Arc<Application>,
    pub bot: Bot,
    pub callback: CallbackQuery,
}

impl CallbackHandler {
    pub async fn public_handle(bot: Bot, callback: CallbackQuery, app: Arc<Application>) -> Result<()> {
        let handler = CallbackHandler { app, bot, callback };
        let data: MemeCallback =
            serde_json::from_str(&handler.callback.data.clone().unwrap_or_else(|| r#"{}"#.to_string()))?;

        let meme = Memes::get_by_id(data.uuid).await.expect("Can't get meme from callback");

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

    pub async fn like(&self, meme: &MemeModel) -> Result<()> {
        let msg = match self.callback.regular_message() {
            Some(msg) => msg,
            None => return Ok(()),
        };

        let user_id = self.callback.from.id.0 as i64;

        if meme.like_exists(user_id).await {
            meme.cancel_like(user_id).await;
        } else {
            meme.like(user_id).await;
        }

        if let Some(counts) = meme.count_all_likes().await {
            self.update_message(meme, msg, counts).await?;
        } else {
            warn!("Can't update counts on meme: {}", &meme.uuid)
        }

        Ok(())
    }

    pub async fn dislike(&self, meme: &MemeModel) -> Result<()> {
        let msg = match self.callback.regular_message() {
            Some(msg) => msg,
            None => return Ok(()),
        };

        let user_id = self.callback.from.id.0 as i64;

        if meme.dislike_exists(user_id).await {
            meme.cancel_dislike(user_id).await;
        } else {
            meme.dislike(user_id).await;
        }

        if let Some(counts) = meme.count_all_likes().await {
            self.update_message(meme, msg, counts).await?;
        } else {
            warn!("Can't update counts on meme: {}", &meme.uuid)
        }

        Ok(())
    }

    pub async fn none(&self, meme: &MemeModel) -> Result<()> {
        let msg = match self.callback.regular_message() {
            Some(msg) => msg,
            None => return Ok(()),
        };

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

    pub async fn delete(&self, meme: &MemeModel) -> Result<()> {
        let msg = match self.callback.regular_message() {
            Some(msg) => msg,
            None => return Ok(()),
        };

        if !self.can_user_interact(meme) {
            self.bot
                .answer_callback_query(&self.callback.id)
                .text("Только тот, кто прислал мем (или админ), может сделать это")
                .show_alert(true)
                .await?;

            return Ok(());
        }

        self.bot.delete_message(msg.chat.id, msg.id).await?;
        self.bot.delete_message(meme.chat_id(), meme.msg_id()).await?;

        meme.remove().await;

        self.bot
            .answer_callback_query(&self.callback.id)
            .text("УдОлено")
            .await?;

        Ok(())
    }

    async fn update_message(&self, meme: &MemeModel, msg: &Message, counts: MemeLikesCountAll) -> Result<()> {
        let meme_markup = MemeMarkup::new(counts.likes, counts.dislikes, meme.uuid);

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

    fn can_user_interact(&self, meme: &MemeModel) -> bool {
        let admins = RedisManager::global().get_chat_admins(self.callback.chat_id().unwrap().0);
        let is_user_admin = admins.contains(&self.callback.from.id.0);

        is_user_admin || (meme.user_id == self.callback.from.id.0 as i64)
    }
}
