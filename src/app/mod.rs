use anyhow::{anyhow, Result};
use envconfig::Envconfig;
use futures::executor::block_on;
use imghash::ImageHash;
use std::{thread::sleep, time::Duration};
use teloxide::{net::Download, prelude::*, types::User};
use tokio::fs::File;
use utils::from_binary_to_hex;

use crate::bot::Bot;
use crate::bot::BotManager;
use crate::database::DBManager;
use crate::redis::RedisManager;

pub mod imghash;
pub mod utils;

pub struct Application {
    pub database: DBManager,
    pub redis: RedisManager,
    pub bot_manager: BotManager,
    pub bot: Bot,
    pub config: Config,
}

#[derive(Envconfig)]
pub struct Config {
    #[envconfig(from = "CARGO_PKG_VERSION", default = "unknown")]
    pub app_version: String,
    #[envconfig(from = "BOT_TOKEN")]
    pub bot_token: String,
    #[envconfig(from = "DATABASE_URL")]
    pub db_url: String,
    #[envconfig(from = "REDIS_URL")]
    pub redis_url: String,
    #[envconfig(from = "CHAT_ID")]
    pub chat_id: i64,
}

impl Application {
    pub fn new() -> Self {
        let config = Config::init_from_env().expect("Can't load config from environment");
        let bot_manager = BotManager::new(&config.bot_token);
        let bot = bot_manager.bot.clone();

        let app = Self {
            database: DBManager::connect(&config.db_url),
            redis: RedisManager::connect(&config.redis_url),
            bot_manager,
            bot,
            config,
        };

        app.register_chat();
        app.check_version();

        app
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

    fn check_version(&self) {
        let chat_id = self.config.chat_id;
        if let Some(redis_version) = self.redis.get_app_version() {
            if redis_version != self.config.app_version {
                block_on(
                    self.bot
                        .send_message(ChatId(chat_id), "😌 Я обновилься!")
                        .send(),
                )
                .expect("Can't send message");
            }
        }

        self.redis.set_app_version(&self.config.app_version);
    }
    fn register_chat(&self) -> bool {
        let chat_id = self.config.chat_id;
        let admins = block_on(self.get_chat_admins(chat_id));

        self.redis.register_chat(chat_id);
        self.redis.set_chat_admins(chat_id, &admins);

        admins.into_iter().for_each(|admin| {
            self.database.add_chat_admin(chat_id, admin);
        });

        true
    }
}
