use commands::PrivateCommand;
use serde::{Deserialize, Serialize};
use teloxide::{
    dispatching::{UpdateFilterExt, UpdateHandler},
    dptree,
    prelude::*,
};

mod admin;
mod commands;
mod messages;

pub fn scheme() -> UpdateHandler<anyhow::Error> {
    dptree::entry().branch(admin::scheme()).branch(
        Update::filter_message()
            .filter(move |m: Message| m.chat.is_private())
            .branch(
                Update::filter_message()
                    .filter_command::<PrivateCommand>()
                    .branch(dptree::case![PrivateCommand::Help].endpoint(commands::help_command)),
            )
            .branch(Update::filter_message().endpoint(messages::handle)),
    )
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub enum PrivateState {
    #[default]
    Idle,
    AdminAddMessage,
}
