use anyhow::{anyhow, Result};
use envconfig::Envconfig;
use futures::executor::block_on;
use imghash::ImageHash;
use std::{thread::sleep, time::Duration};
use teloxide::prelude::*;
use utils::from_binary_to_hex;

use crate::bot::{BotConfig, BotManager};
use crate::database::entity::{memes, prelude::*};
use crate::redis::RedisManager;

pub mod imghash;
pub mod utils;

pub struct SimilarMeme {
    pub percent: i64,
    pub meme: Option<memes::Model>,
}

#[derive(Clone, Debug)]
pub struct Application {
    pub redis: RedisManager,
    pub config: Config,
    pub bot: BotManager,
}

#[derive(Envconfig, Clone, Debug)]
pub struct Config {
    #[envconfig(from = "BOT_VERSION", default = "unknown")]
    pub app_version: String,
    #[envconfig(from = "DATABASE_URL")]
    pub db_url: String,
    #[envconfig(from = "REDIS_URL")]
    pub redis_url: String,
    #[envconfig(nested = true)]
    pub bot: BotConfig,
}

impl Application {
    pub fn new() -> Self {
        let config = Config::init_from_env().expect("Can't load config from environment");

        Self {
            redis: RedisManager::connect(&config.redis_url),
            bot: BotManager::new(&config.bot),
            config,
        }
    }

    pub async fn generate_hashes(&self, file_id: &str) -> Result<(Option<String>, Option<String>)> {
        let path = self.bot.download_file(file_id).await?;

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

    pub async fn get_similar_meme(short_hash: &str, long_hash: &str) -> SimilarMeme {
        let mut s_meme = SimilarMeme { percent: 0, meme: None };

        let similar_memes = Memes::get_by_short_hash(short_hash).await;

        similar_memes.into_iter().for_each(|meme| {
            let meme_hash = meme.long_hash.clone().unwrap_or_default();

            if meme_hash.len() == long_hash.len() {
                let percent = ImageHash::compare_hashes(
                    &utils::from_hex_to_binary(long_hash),
                    &utils::from_hex_to_binary(&meme_hash),
                );

                if percent > 93f64 && percent < 99f64 {
                    if percent as i64 > s_meme.percent {
                        s_meme = SimilarMeme {
                            percent: percent as i64,
                            meme: Some(meme),
                        };
                    }
                } else if percent >= 99f64 {
                    s_meme = SimilarMeme {
                        percent: 100,
                        meme: Some(meme),
                    };
                }
            }
        });

        s_meme
    }

    pub fn check_version(&self) {
        let chat_id = self.config.bot.chat_id;
        if let Some(redis_version) = self.redis.get_app_version() {
            if redis_version != self.config.app_version {
                block_on(self.bot.get().send_message(ChatId(chat_id), "😌 Я обновилься!").send())
                    .expect("Can't send message");
            }
        }

        self.redis.set_app_version(&self.config.app_version);
    }

    pub fn register_chat(&self) -> bool {
        let chat_id = self.config.bot.chat_id;
        let admins = block_on(self.bot.get_chat_admins(chat_id));

        self.redis.register_chat(chat_id);
        self.redis.set_chat_admins(chat_id, &admins);

        ChatAdmins::add_admins(chat_id, &admins);

        true
    }
}
