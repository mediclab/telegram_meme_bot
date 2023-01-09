extern crate dotenv;

mod bot;
mod database;

use dotenv::dotenv;
use teloxide::prelude::*;
use teloxide::types::Me;
use std::{sync::Arc, env};
use clap::{ArgMatches, Command, Arg};

use database::manager::DBManager;
use bot::messages::MessagesHandler;
use bot::callbacks::CallbackHandler;
use bot::commands::{CommandsHandler, Command as BotCommands};

pub struct Application {
    pub database: DBManager,
    pub bot: Me,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    pretty_env_logger::init();

    let args = cli();

    let bot = Bot::from_env();
    let state = Arc::new(
        Application {
            database: DBManager::connect(
                env::var("DATABASE_URL").expect("DATABASE_URL must be set")
            ),
            bot: bot.get_me().await.expect("Can't get bot information"),
        }
    );

    if let Some(_) = args.get_one::<String>("start") {
        let handler =
            dptree::entry()
                .branch(Update::filter_message().filter_command::<BotCommands>().endpoint(CommandsHandler::handle))
                .branch(Update::filter_message().endpoint(MessagesHandler::handle))
                .branch(Update::filter_callback_query().endpoint(CallbackHandler::handle))
            ;

        println!("Starting dispatch...");

        Dispatcher::builder(bot, handler)
            .dependencies(dptree::deps![Arc::clone(&state)])
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await
        ;
    }
}

fn cli() -> ArgMatches {
    Command::new("MemeBot")

        .arg(Arg::new("start").long("start")
            .value_parser(["false", "true"])
            .default_value("false")
            .num_args(0)
            .default_missing_value("true")
            .help("Starts the bot daemon")
        )

        .author("Medic84")
        .about("Meme telegram bot for chats")
        .get_matches()
}
