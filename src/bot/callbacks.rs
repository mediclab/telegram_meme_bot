use std::error::Error;
use std::sync::Arc;
use futures::executor::block_on;

use crate::Application;
use crate::bot::markups::*;
use crate::database::{
    models::Meme,
    repository::*
};

use teloxide::{
    prelude::*,
    types::MessageId
};

pub struct CallbackHandler {
    pub app: Arc<Application>,
    pub bot: Bot,
    pub callback: CallbackQuery
}

impl CallbackHandler {
    pub async fn handle(bot: Bot, callback: CallbackQuery, app: Arc<Application>) -> Result<(), Box<dyn Error + Send + Sync>> {
        let handler = CallbackHandler { app, bot, callback };

        let data: MemeCallback = serde_json::from_str(
            &handler.callback.data.clone().unwrap_or(r#"{}"#.to_string())
        )?;

        let meme = MemeRepository::new(handler.app.database.clone()).get(&data.uuid)?;

        match data.op {
            CallbackOperations::Like => {
                handler.like(&meme)?;
            }
            CallbackOperations::Dislike => {
                handler.dislike(&meme)?;
            }
            CallbackOperations::Delete => {
                handler.delete(&meme);
            }
            CallbackOperations::None => {
                handler.none(&meme);
            }
        }

        Ok(())
    }

    pub fn like(&self, meme: &Meme) -> Result<(), Box<dyn Error + Send + Sync>> {
        let msg = self.callback.message.clone().unwrap();
        let repository = MemeLikeRepository::new(self.app.database.clone());

        let t = repository.like(self.callback.from.id.0 as i64, &meme.uuid);
        let likes = (repository.count_likes(&meme.uuid), repository.count_dislikes(&meme.uuid));

        self.update_message(meme, &msg, likes)
    }

    pub fn dislike(&self, meme: &Meme) -> Result<(), Box<dyn Error + Send + Sync>> {
        let msg = self.callback.message.clone().unwrap();
        let repository = MemeLikeRepository::new(self.app.database.clone());

        let t = repository.dislike(self.callback.from.id.0 as i64, &meme.uuid);
        let likes = (repository.count_likes(&meme.uuid), repository.count_dislikes(&meme.uuid));

        self.update_message(meme, &msg, likes)
    }

    pub fn none(&self, meme: &Meme) {
        let msg = self.callback.message.as_ref().unwrap();

        if meme.user_id != self.callback.from.id.0 as i64 {
            block_on(async {
                self.bot
                    .answer_callback_query(&self.callback.id)
                    .text("Только пользователь отправивший мем, может сделать это")
                    .show_alert(true)
                    .await.expect("Can't answer callback query")
            });

            return;
        }

        block_on(async {
            self.bot
                .delete_message(msg.chat.id, msg.id)
                .await.expect("Can't delete message")
        });
    }

    pub fn delete(&self, meme: &Meme) {
        self.none(meme);

        block_on(async {
            self.bot
                .delete_message(
                    ChatId { 0: meme.chat_id },
                    MessageId { 0: meme.msg_id.unwrap() as i32 }
                )
                .await.expect("Can't delete meme");
        });
    }

    fn update_message(&self, meme: &Meme, msg: &Message, counts: (i64, i64)) -> Result<(), Box<dyn Error + Send + Sync>> {
        let (likes, dislikes) = counts;
        let meme_markup = MemeMarkup::new(likes, dislikes, meme.uuid);

        let req = self.bot
            .edit_message_reply_markup(msg.chat.id, msg.id)
            .reply_markup(meme_markup.get_markup());

        block_on(req.send())?;

        Ok(())
    }
}
