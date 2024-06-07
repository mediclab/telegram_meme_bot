use crate::database::entity::prelude::ChatAdmins;
use commands::AdminCommand;
use teloxide::dispatching::{UpdateFilterExt, UpdateHandler};
use teloxide::dptree;
use teloxide::prelude::*;

mod commands;

pub fn scheme() -> UpdateHandler<anyhow::Error> {
    dptree::entry().branch(
        Update::filter_message()
            .filter(move |m: Message| m.chat.is_private() && ChatAdmins::is_user_admin(m.from().unwrap().id.0 as i64))
            .branch(
                Update::filter_message()
                    .filter_command::<AdminCommand>()
                    .branch(dptree::case![AdminCommand::Help].endpoint(commands::help_command))
                    .branch(dptree::case![AdminCommand::Message(x)].endpoint(commands::message_command)),
            ),
    )
}
