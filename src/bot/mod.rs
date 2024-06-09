use anyhow::Result;
use envconfig::Envconfig;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use teloxide::{
    adaptors::DefaultParseMode,
    dispatching::{
        dialogue::{serializer::Json, RedisStorage},
        Dispatcher,
    },
    dptree,
    net::Download,
    prelude::*,
    types::{Chat, ParseMode, User},
};
use tokio::fs::File;

mod private;
mod public;
pub mod statistics;
pub mod types;

pub type Bot = DefaultParseMode<teloxide::Bot>;
type BotDialogue = Dialogue<State, RedisStorage<Json>>;

pub static INSTANCE: OnceCell<BotManager> = OnceCell::new();

#[derive(Envconfig, Clone, Debug)]
pub struct BotConfig {
    #[envconfig(from = "CHAT_ID")]
    pub chat_id: i64,
    #[envconfig(from = "BOT_TOKEN")]
    pub bot_token: String,
}

#[derive(Clone, Debug)]
pub struct BotManager {
    bot: Bot,
    chat_id: i64,
}

impl BotManager {
    pub fn new(config: &BotConfig) -> Self {
        Self {
            bot: teloxide::Bot::new(&config.bot_token).parse_mode(ParseMode::Html),
            chat_id: config.chat_id,
        }
    }

    pub fn global() -> &'static BotManager {
        INSTANCE.get().expect("Can't get bot")
    }

    pub async fn get_chat_user(&self, user_id: i64) -> User {
        let member = self
            .bot
            .get_chat_member(ChatId(self.chat_id), UserId(user_id as u64))
            .await
            .expect("Can't get chat member");

        member.user
    }

    pub async fn download_file(&self, file_id: &str) -> Result<String> {
        let photo = self.bot.get_file(file_id).await?;
        let path = format!("/tmp/{}", uuid::Uuid::new_v4());
        let mut file = File::create(&path).await?;

        self.bot.download_file(&photo.path, &mut file).await?;

        Ok(path)
    }

    pub async fn get_chat_admins(&self, chat_id: i64) -> Vec<u64> {
        if let Ok(res) = self.bot.get_chat_administrators(ChatId(chat_id)).await {
            res.iter().map(|m| m.user.id.0).collect::<Vec<u64>>()
        } else {
            Vec::default()
        }
    }

    pub async fn dispatch(&self, deps: DependencyMap) {
        Dispatcher::builder(
            self.bot.clone(),
            dptree::entry().branch(public::scheme()).branch(private::scheme()),
        )
        .dependencies(deps)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await
    }

    pub fn get(&self) -> &Bot {
        &self.bot
    }

    fn filter_messages(ch: &Chat, chat_id: i64) -> bool {
        (ch.is_group() || ch.is_supergroup()) && ch.id.0 == chat_id
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub enum State {
    #[default]
    Idle,
    Private(private::PrivateState),
}
