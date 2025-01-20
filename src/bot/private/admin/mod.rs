use crate::bot::{private::PrivateState, State};
use crate::database::entity::prelude::ChatAdmins;
use teloxide::dptree;
use teloxide::{
    dispatching::{
        dialogue::{serializer::Json, RedisStorage},
        UpdateFilterExt, UpdateHandler,
    },
    prelude::*,
};

mod callbacks;
mod commands;
mod messages;
mod types;

pub fn scheme() -> UpdateHandler<anyhow::Error> {
    dptree::entry()
        .branch(
            Update::filter_message()
                .filter(move |m: Message| m.chat.is_private() && ChatAdmins::is_user_admin(m.from.unwrap().id.0 as i64))
                .enter_dialogue::<Message, RedisStorage<Json>, State>()
                .branch(
                    Update::filter_message()
                        .filter_command::<commands::AdminCommand>()
                        .branch(dptree::case![commands::AdminCommand::Help].endpoint(commands::help_command))
                        .branch(dptree::case![commands::AdminCommand::Message(x)].endpoint(commands::message_command))
                        .branch(
                            dptree::case![commands::AdminCommand::AddMessage].endpoint(commands::add_message_command),
                        ),
                )
                .branch(
                    Update::filter_message().branch(
                        dptree::case![State::Private(x)].branch(
                            dptree::case![PrivateState::AdminAddMessage].endpoint(messages::add_message_handle),
                        ),
                    ),
                ),
        )
        .branch(
            Update::filter_callback_query()
                .filter(move |c: CallbackQuery| {
                    c.message.unwrap().chat().is_private() && ChatAdmins::is_user_admin(c.from.id.0 as i64)
                })
                .enter_dialogue::<CallbackQuery, RedisStorage<Json>, State>()
                .endpoint(callbacks::handle),
        )
}
