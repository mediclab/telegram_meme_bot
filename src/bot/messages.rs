use std::sync::Arc;

use anyhow::Result;
use teloxide::{
    prelude::*,
    types::{InputFile, MessageKind, PhotoSize, Video},
};

use crate::bot::markups::*;
use crate::database::models::{AddMeme, AddUser, Meme};
use crate::utils as Utils;
use crate::Application;
use crate::bot::Bot;

pub struct MessagesHandler {
    pub app: Arc<Application>,
    pub bot: Bot,
    pub msg: Message,
}

impl MessagesHandler {
    pub async fn handle(bot: Bot, msg: Message, app: Arc<Application>) -> Result<()> {
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

    pub async fn common(&self) -> Result<()> {
        // If This is forwarded message - nothing to do.
        if self.msg.forward().is_some() {
            return Ok(());
        }

        // If caption contains "nomeme" - nothing to do.
        if self
            .msg
            .caption()
            .unwrap_or("")
            .to_lowercase()
            .contains("nomem")
        {
            return Ok(());
        }

        if self.msg.from().is_none() {
            warn!("Anonimous user detected");

            return Ok(());
        }

        if self.msg.photo().is_some() || self.msg.video().is_some() {
            if !self.app.redis.is_chat_registered(self.msg.chat.id.0) {
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

    pub async fn newbie(&self) -> Result<()> {
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
            let _ = self.app.database.add_user(&AddUser::new_from_tg(user));
        });

        Ok(())
    }

    pub async fn left(&self) -> Result<()> {
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

        self.app.database.delete_user(user.id.0 as i64);

        Ok(())
    }

    async fn photo_handle(&self, photos: &[PhotoSize]) -> Result<()> {
        let user = self.msg.from().unwrap();
        let user_text = Utils::get_user_text(user);

        let (hash, hash_min) = match Utils::generate_hashes(&self.bot, &photos[0].file.id).await {
            Ok(res) => res,
            Err(e) => {
                warn!("Can't generate hashes. Error: {e}");

                (None, None)
            }
        };
        let mut s_meme: (i64, Option<Meme>) = (0, None);

        if hash.is_some() && hash_min.is_some() {
            let hash_min = hash_min.clone().unwrap();
            let similar_memes = self
                .app
                .database
                .get_memes_by_short_hash(&hash_min)
                .unwrap_or_default();

            similar_memes.into_iter().for_each(|meme| {
                let hash = hash.clone().unwrap();
                let meme_hash = meme.long_hash.clone().unwrap_or_default();

                if meme_hash.len() == hash.len() {
                    let percent = Utils::compare_hashes(
                        &Utils::from_hex_to_binary(&hash),
                        &Utils::from_hex_to_binary(&meme_hash),
                    );

                    if percent > 90f64 && percent < 99f64 {
                        if percent as i64 > s_meme.0 {
                            s_meme = (percent as i64, Some(meme));
                        }
                    } else if percent >= 99f64 {
                        s_meme = (100, Some(meme));
                    }
                }
            });
        }

        self.bot
            .delete_message(self.msg.chat.id, self.msg.id)
            .await?;

        if s_meme.0 == 100 {
            let meme = s_meme.1.unwrap();
            let messages =
                Utils::Messages::load(include_str!("../../messages/meme_already_exists.in"));

            self.bot
                .send_message(
                    self.msg.chat.id,
                    messages.random().replace("{user_name}", &user_text),
                )
                .reply_to_message_id(meme.msg_id())
                .await?;

            return Ok(());
        }

        let meme = self
            .app
            .database
            .add_meme(&AddMeme::new_from_tg(&self.msg, &hash, &hash_min))
            .expect("Can't add photo meme");

        let markup = MemeMarkup::new(0, 0, meme.uuid);
        let bot_msg = self
            .bot
            .send_photo(self.msg.chat.id, InputFile::file_id(&photos[0].file.id))
            .caption(format!("Оцените мем {user_text}"))
            .reply_markup(markup.get_markup())
            .await?;

        self.app
            .database
            .replace_meme_msg_id(&meme.uuid, bot_msg.id.0 as i64);

        if s_meme.0 > 0 {
            let meme = s_meme.1.unwrap();
            let messages = Utils::Messages::load(include_str!("../../messages/similar_meme.in"));

            self.bot
                .send_message(
                    self.msg.chat.id,
                    messages
                        .random()
                        .replace("{user_name}", &user_text)
                        .replace(
                            "{percent}",
                            &Utils::pluralize(s_meme.0, ("процент", "процента", "процентов")),
                        ),
                )
                .reply_to_message_id(meme.msg_id())
                .reply_markup(
                    DeleteMarkup::new(meme.uuid)
                        .set_ok_text(&format!(
                            "{} Упс, действительно было...",
                            emojis::get_by_shortcode("wastebasket").unwrap().as_str()
                        ))
                        .set_none_text(&format!(
                            "{} Это точно свежак! ",
                            emojis::get_by_shortcode("x").unwrap().as_str()
                        ))
                        .get_markup(),
                )
                .await?;
        }

        Ok(())
    }

    async fn video_handle(&self, video: &Video) -> Result<()> {
        let user = self.msg.from().unwrap();
        let user_text = Utils::get_user_text(user);

        let meme = self
            .app
            .database
            .add_meme(&AddMeme::new_from_tg(
                &self.msg,
                &None as &Option<String>,
                &None as &Option<String>,
            ))
            .expect("Can't add video meme");

        self.bot
            .delete_message(self.msg.chat.id, self.msg.id)
            .await?;

        let markup = MemeMarkup::new(0, 0, meme.uuid);
        let bot_msg = self
            .bot
            .send_video(self.msg.chat.id, InputFile::file_id(&video.file.id))
            .caption(format!("Оцените видео-мем {user_text}"))
            .reply_markup(markup.get_markup())
            .await?;

        self.app
            .database
            .replace_meme_msg_id(&meme.uuid, bot_msg.id.0 as i64);

        Ok(())
    }
}
