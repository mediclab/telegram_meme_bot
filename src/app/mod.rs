use anyhow::{anyhow, Result};
use envconfig::Envconfig;
use futures::executor::block_on;
use imghash::ImageHash;
use std::{thread::sleep, time::Duration};
use teloxide::prelude::*;
use utils::from_binary_to_hex;

use crate::bot::{BotConfig, BotManager};
use crate::database::entity::prelude::*;
use crate::database::DBManager;
use crate::nats::{NatsConfig, NatsManager};
use crate::redis::RedisManager;

pub mod imghash;
pub mod utils;

#[derive(Clone, Debug)]
pub struct Application {
    pub database: DBManager,
    pub redis: RedisManager,
    pub nats: NatsManager,
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
    pub nats: NatsConfig,
    #[envconfig(nested = true)]
    pub bot: BotConfig,
}

impl Application {
    pub fn new() -> Self {
        let config = Config::init_from_env().expect("Can't load config from environment");

        Self {
            database: DBManager::connect(&config.db_url),
            redis: RedisManager::connect(&config.redis_url),
            bot: BotManager::new(&config.bot),
            nats: NatsManager::new(&config.nats),
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
