use crate::bot::callbacks::CallbackHandler;
use crate::bot::commands::{CommandsHandler, PrivateCommand, PublicCommand};
use crate::bot::messages::MessagesHandler;
use anyhow::Result;
use envconfig::Envconfig;
use teloxide::adaptors::DefaultParseMode;
use teloxide::dispatching::{Dispatcher, HandlerExt, UpdateFilterExt};
use teloxide::dptree;
use teloxide::net::Download;
use teloxide::prelude::{
    Bot as TxBot, ChatId, ChatMemberUpdated, DependencyMap, Message, Requester, RequesterExt, Update, UserId,
};
use teloxide::types::{Chat, ParseMode, User};
use tokio::fs::File;

pub mod callbacks;
pub mod commands;
pub mod markups;
pub mod messages;
pub mod statistics;

pub type Bot = DefaultParseMode<teloxide::Bot>;

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
            bot: TxBot::new(&config.bot_token).parse_mode(ParseMode::Html),
            chat_id: config.chat_id,
        }
    }

    pub async fn get_chat_user(&self, chat_id: i64, user_id: i64) -> User {
        let member = self
            .bot
            .get_chat_member(ChatId(chat_id), UserId(user_id as u64))
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
        let chat_id = self.chat_id;
        let handler = dptree::entry()
            .branch(
                Update::filter_message()
                    .filter(|m: Message| m.chat.is_private())
                    .filter_command::<PrivateCommand>()
                    .endpoint(CommandsHandler::private_handle),
            )
            .branch(
                Update::filter_chat_member()
                    .filter(move |cm: ChatMemberUpdated| BotManager::filter_messages(&cm.chat, chat_id))
                    .endpoint(MessagesHandler::chat_member_handle),
            )
            .branch(
                Update::filter_message()
                    .filter(move |m: Message| BotManager::filter_messages(&m.chat, chat_id))
                    .filter_command::<PublicCommand>()
                    .endpoint(CommandsHandler::public_handle),
            )
            .branch(
                Update::filter_message()
                    .filter(|m: Message| m.chat.is_private())
                    .endpoint(MessagesHandler::private_handle),
            )
            .branch(
                Update::filter_message()
                    .filter(move |m: Message| BotManager::filter_messages(&m.chat, chat_id))
                    .endpoint(MessagesHandler::public_handle),
            )
            .branch(Update::filter_callback_query().endpoint(CallbackHandler::handle));

        Dispatcher::builder(self.bot.clone(), handler)
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
