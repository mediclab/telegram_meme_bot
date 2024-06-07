use commands::PrivateCommand;
use teloxide::dispatching::{UpdateFilterExt, UpdateHandler};
use teloxide::dptree;
use teloxide::prelude::*;

mod admin;
mod commands;

pub fn scheme() -> UpdateHandler<anyhow::Error> {
    dptree::entry().branch(admin::scheme()).branch(
        Update::filter_message()
            .filter(move |m: Message| m.chat.is_private())
            .branch(
                Update::filter_message()
                    .filter_command::<PrivateCommand>()
                    .branch(dptree::case![PrivateCommand::Help].endpoint(commands::help_command)),
            ),
    )
}
