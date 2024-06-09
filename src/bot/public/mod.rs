use crate::bot::BotManager;
use commands::PublicCommand;
use teloxide::{
    dispatching::{UpdateFilterExt, UpdateHandler},
    dptree,
    prelude::*,
};

mod callbacks;
mod commands;
mod markups;
mod messages;
mod types;

pub fn scheme() -> UpdateHandler<anyhow::Error> {
    let chat_id = BotManager::global().chat_id;
    dptree::entry()
        .branch(
            Update::filter_message()
                .filter(move |m: Message| BotManager::filter_messages(&m.chat, chat_id))
                .branch(
                    Update::filter_message()
                        .filter_command::<PublicCommand>()
                        .branch(dptree::case![PublicCommand::Accordion].endpoint(commands::accordion_command))
                        .branch(dptree::case![PublicCommand::F].endpoint(commands::f_command))
                        .branch(dptree::case![PublicCommand::Stats].endpoint(commands::stats_command))
                        .branch(dptree::case![PublicCommand::UnMeme].endpoint(commands::unmeme_command))
                        .branch(dptree::case![PublicCommand::Help].endpoint(commands::help_command)),
                )
                .branch(Update::filter_message().endpoint(messages::common)),
        )
        .branch(
            Update::filter_chat_member()
                .filter(move |cm: ChatMemberUpdated| BotManager::filter_messages(&cm.chat, chat_id))
                .endpoint(messages::chat_member_handle),
        )
        .branch(
            Update::filter_callback_query()
                .filter(move |c: CallbackQuery| {
                    c.message.is_some() && BotManager::filter_messages(&c.message.unwrap().chat, chat_id)
                })
                .endpoint(callbacks::CallbackHandler::public_handle),
        )
}
