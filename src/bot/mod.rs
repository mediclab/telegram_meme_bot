use futures::executor::block_on;

use teloxide::{
    adaptors::DefaultParseMode,
    prelude::*,
    types::{Chat, Me, ParseMode},
    Bot as TxBot,
};

use crate::bot::{
    callbacks::CallbackHandler,
    commands::{CommandsHandler, PrivateCommand, PublicCommand},
    messages::MessagesHandler,
};

pub mod callbacks;
pub mod commands;
pub mod markups;
pub mod messages;
pub mod top;

pub type Bot = DefaultParseMode<teloxide::Bot>;

#[derive(Clone)]
pub struct BotManager {
    pub bot: Bot,
    pub me: Me,
}

impl BotManager {
    pub fn new(token: &str) -> Self {
        let bot = TxBot::new(token).parse_mode(ParseMode::Html);
        let me = block_on(bot.get_me().send()).expect("Can't get Me info");

        Self { bot, me }
    }

    pub async fn dispatch(&self, deps: DependencyMap, chat_id: i64) {
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

    fn filter_messages(ch: &Chat, chat_id: i64) -> bool {
        (ch.is_group() || ch.is_supergroup()) && ch.id.0 == chat_id
    }
}
