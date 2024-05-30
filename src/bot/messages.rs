use std::sync::Arc;

use anyhow::Result;
use teloxide::types::ChatMemberKind;
use teloxide::{
    prelude::*,
    types::{InputFile, MessageKind, PhotoSize, Video},
};

use crate::app::{imghash::ImageHash, utils as Utils, Application};
use crate::bot::{markups::*, Bot};
use crate::database::entity::memes;
use crate::database::entity::prelude::{Memes, Users};

pub struct MessagesHandler {
    pub app: Arc<Application>,
    pub bot: Bot,
    pub msg: Message,
}

impl MessagesHandler {
    pub async fn public_handle(bot: Bot, msg: Message, app: Arc<Application>) -> Result<()> {
        let handler = MessagesHandler { app, bot, msg };

        match &handler.msg.kind {
            MessageKind::Common(_) => {
                handler.common().await?;
            }
            MessageKind::NewChatMembers(_) | MessageKind::LeftChatMember(_) => {
                handler.bot.delete_message(handler.msg.chat.id, handler.msg.id).await?;
            }
            _ => {}
        };

        Ok(())
    }

    pub async fn private_handle() -> Result<()> {
        Ok(())
    }

    pub async fn chat_member_handle(bot: Bot, cm: ChatMemberUpdated) -> Result<()> {
        let member = cm.new_chat_member;
        match member.kind {
            ChatMemberKind::Member => {
                let messages = Utils::Messages::load(include_str!("../../messages/newbie.in"));
                bot.send_message(
                    cm.chat.id,
                    messages
                        .random()
                        .replace("{user_name}", &Utils::get_user_text(&member.user)),
                )
                .await?;

                Users::add(&member.user).await;
            }
            ChatMemberKind::Left | ChatMemberKind::Banned(_) => {
                let messages = Utils::Messages::load(include_str!("../../messages/left.in"));
                bot.send_message(
                    cm.chat.id,
                    messages
                        .random()
                        .replace("{user_name}", &Utils::get_user_text(&member.user)),
                )
                .await?;

                Users::delete(member.user.id.0 as i64).await;
            }
            _ => {}
        }

        Ok(())
    }

    pub async fn common(&self) -> Result<()> {
        // If This is forwarded message - nothing to do.
        if self.msg.forward().is_some() {
            return Ok(());
        }

        // If caption contains "nomeme" - nothing to do.
        if self.msg.caption().unwrap_or("").to_lowercase().contains("nomem") {
            return Ok(());
        }

        if self.msg.from().is_none() {
            warn!("Anonimous user detected");

            return Ok(());
        }

        if self.msg.photo().is_some() || self.msg.video().is_some() {
            if !self.app.redis.is_chat_registered(self.msg.chat.id.0) {
                warn!("Chat {} is not registered", self.msg.chat.id.0);

                return Ok(());
            }

            if let Some(photos) = self.msg.photo() {
                self.photo_handle(photos).await?;
            }

            if let Some(video) = self.msg.video() {
                self.video_handle(video).await?
            }
        }

        Ok(())
    }

    async fn photo_handle(&self, photos: &[PhotoSize]) -> Result<()> {
        let user = self.msg.from().unwrap();
        let user_text = Utils::get_user_text(user);

        let hash_result = self.app.generate_hashes(&photos[0].file.id).await;
        let (hash, hash_min) = hash_result.unwrap_or_else(|e| {
            warn!("Can't generate hashes. Error: {e}");

            (None, None)
        });
        let mut s_meme: (i64, Option<memes::Model>) = (0, None);

        if hash.is_some() && hash_min.is_some() {
            let hash_min = hash_min.clone().unwrap();
            let similar_memes = Memes::get_by_short_hash(&hash_min).await;

            similar_memes.into_iter().for_each(|meme| {
                let hash = hash.clone().unwrap();
                let meme_hash = meme.long_hash.clone().unwrap_or_default();

                if meme_hash.len() == hash.len() {
                    let percent = ImageHash::compare_hashes(
                        &Utils::from_hex_to_binary(&hash),
                        &Utils::from_hex_to_binary(&meme_hash),
                    );

                    if percent > 93f64 && percent < 99f64 {
                        if percent as i64 > s_meme.0 {
                            s_meme = (percent as i64, Some(meme));
                        }
                    } else if percent >= 99f64 {
                        s_meme = (100, Some(meme));
                    }
                }
            });
        }

        self.bot.delete_message(self.msg.chat.id, self.msg.id).await?;

        if s_meme.0 == 100 {
            let meme = s_meme.1.unwrap();
            let messages = Utils::Messages::load(include_str!("../../messages/meme_already_exists.in"));

            self.bot
                .send_message(self.msg.chat.id, messages.random().replace("{user_name}", &user_text))
                .reply_to_message_id(meme.msg_id())
                .await?;

            return Ok(());
        }

        let meme = match Memes::add(&self.msg, &hash, &hash_min).await {
            None => {
                warn!("Meme is empty after insert!");
                return Ok(());
            }
            Some(m) => m,
        };

        let markup = MemeMarkup::new(0, 0, meme.uuid);
        let caption = if let Some(caption) = self.msg.caption() {
            format!("\n\nС подписью: {caption}")
        } else {
            String::new()
        };

        let bot_msg = self
            .bot
            .send_photo(self.msg.chat.id, InputFile::file_id(&photos[0].file.id))
            .caption(format!("Оцените мем {user_text}{caption}"))
            .reply_markup(markup.get_markup())
            .await?;

        meme.replace_msg_id(bot_msg.id.0 as i64).await;

        if s_meme.0 > 0 {
            let messages = Utils::Messages::load(include_str!("../../messages/similar_meme.in"));

            self.bot
                .send_message(
                    self.msg.chat.id,
                    messages.random().replace("{user_name}", &user_text).replace(
                        "{percent}",
                        &Utils::Messages::pluralize(s_meme.0, ("процент", "процента", "процентов")),
                    ),
                )
                .reply_to_message_id(s_meme.1.unwrap().msg_id())
                .reply_markup(
                    DeleteMarkup::new(meme.uuid)
                        .set_ok_text("🗑 Упс, действительно, было...")
                        .set_none_text("❌ Это точно свежак!")
                        .get_markup(),
                )
                .await?;
        }

        Ok(())
    }

    async fn video_handle(&self, video: &Video) -> Result<()> {
        let user = self.msg.from().unwrap();
        let user_text = Utils::get_user_text(user);

        let meme = match Memes::add(&self.msg, &None, &None).await {
            None => {
                warn!("Meme is empty after insert!");
                return Ok(());
            }
            Some(m) => m,
        };

        self.bot.delete_message(self.msg.chat.id, self.msg.id).await?;

        let markup = MemeMarkup::new(0, 0, meme.uuid);
        let caption = if let Some(caption) = self.msg.caption() {
            format!("\n\nС подписью: {caption}")
        } else {
            String::new()
        };

        let bot_msg = self
            .bot
            .send_video(self.msg.chat.id, InputFile::file_id(&video.file.id))
            .caption(format!("Оцените видео-мем {user_text}{caption}"))
            .reply_markup(markup.get_markup())
            .await?;

        meme.replace_msg_id(bot_msg.id.0 as i64).await;

        Ok(())
    }
}
