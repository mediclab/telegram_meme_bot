use std::{thread::sleep, time::Duration};

use anyhow::{anyhow, Result};
use teloxide::types::User;
use teloxide::{net::Download, prelude::*, types::PhotoSize};
use tokio::fs::File;

use imghash::ImageHash;
use utils::from_binary_to_hex;

use crate::bot;
use crate::database::models::AddUser;
use crate::database::DBManager;
use crate::redis::RedisManager;

pub mod imghash;
pub mod utils;

pub struct Application {
    pub database: DBManager,
    pub redis: RedisManager,
    pub bot: bot::Bot,
    pub version: String,
}

impl Application {
    pub async fn update_hashes(&self) -> Result<()> {
        let memes = self.database.get_memes_without_hashes()?;

        info!("Count updating memes hashes = {}", memes.len());

        for meme in &memes {
            info!("Start updating hashes for = {}", &meme.uuid);
            let json: Vec<PhotoSize> = match serde_json::from_value(meme.photos.clone().unwrap()) {
                Ok(res) => res,
                Err(_) => {
                    error!("Can't deserialize photos of meme = {}", &meme.uuid);
                    continue;
                }
            };

            let (hash, hash_min) = match self.generate_hashes(&json[0].file.id).await {
                Ok(res) => res,
                Err(_) => (None, None),
            };

            if hash_min.is_some() && hash.is_some() {
                self.database
                    .add_meme_hashes(&meme.uuid, &hash.unwrap(), &hash_min.unwrap());

                info!("Updated hashes for = {}", &meme.uuid);
            } else {
                error!("Failed to update hashes for = {}", &meme.uuid);
            }

            sleep(Duration::from_secs(1));
        }

        Ok(())
    }

    pub async fn generate_hashes(
        &self,
        file_id: &String,
    ) -> Result<(Option<String>, Option<String>)> {
        let photo = self.bot.get_file(file_id).await?;
        let path = format!("/tmp/{}", uuid::Uuid::new_v4());
        let mut file = File::create(&path).await?;

        self.bot.download_file(&photo.path, &mut file).await?;

        sleep(Duration::from_millis(50)); // Sometimes downloading is very fast
        debug!("Filesize {path} is = {}", std::fs::metadata(&path)?.len());

        let cv_image = ImageHash::new(&path).grayscale();
        let hash = cv_image.clone().resize(32).threshold().hash();
        let hash_min = cv_image.resize(4).threshold().hash();

        std::fs::remove_file(&path).unwrap_or_default();

        if hash.is_none() || hash_min.is_none() {
            return Err(anyhow!("Error in opencv hashing"));
        }

        Ok((
            Some(from_binary_to_hex(&hash.unwrap())),
            Some(from_binary_to_hex(&hash_min.unwrap())),
        ))
    }

    pub async fn update_users(&self, chat_id: i64) -> Result<()> {
        let uids = self.database.get_users_ids_not_in_table()?;

        info!("Count updating users = {}", uids.len());

        for uid in &uids {
            info!("Sending request for user id = {uid}");
            let res = self
                .bot
                .get_chat_member(ChatId(chat_id), UserId(*uid as u64))
                .await;

            let member = match res {
                Ok(m) => m,
                Err(e) => {
                    error!("User not found: {e}");

                    continue;
                }
            };

            info!("Add user {uid} to database ({})", member.user.full_name());

            let _ = self.database.add_user(&AddUser::new_from_tg(&member.user));

            sleep(Duration::from_secs(1));
        }

        Ok(())
    }

    pub async fn get_chat_admins(&self, chat_id: i64) -> Vec<u64> {
        let admins = self.bot.get_chat_administrators(ChatId(chat_id)).await;

        if admins.is_err() {
            return Vec::default();
        }

        admins
            .unwrap()
            .iter()
            .map(|m| m.user.id.0)
            .collect::<Vec<u64>>()
    }

    pub async fn get_chat_user(&self, chat_id: i64, user_id: u64) -> Result<User> {
        let member = self
            .bot
            .get_chat_member(ChatId(chat_id), UserId(user_id))
            .await;

        Ok(member.expect("Can't get chat member").user)
    }
}
