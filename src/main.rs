extern crate dotenv;

mod bot;
mod database;

use dotenv::dotenv;
use teloxide::prelude::*;
use clap::Parser;
use teloxide::types::Me;
use std::{sync::Arc, env};

use database::manager::DBManager;
use bot::messages as BotMessages;
use bot::callbacks as BotCallbacks;
use bot::commands as BotCommands;

pub struct Application {
    pub database: DBManager,
    pub bot: Me,
}

#[derive(Parser, Default, Debug)]
#[clap(author="Medic84", version, about="Meme telegram bot for chats")]
struct Arguments {
    /// Run the bot as daemon
    bot: bool,
    max_depth: usize,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    pretty_env_logger::init();

    //let args = Arguments::parse();

    let bot = Bot::from_env();
    let state = Arc::new(
        Application {
            database: DBManager::connect(
                env::var("DATABASE_URL").expect("DATABASE_URL must be set")
            ),
            bot: bot.get_me().await.expect("Can't get bot information")
        }
    );

    let handler = 
        dptree::entry()
        .branch(Update::filter_message().filter_command::<BotCommands::Command>().endpoint(BotCommands::handle))
        .branch(Update::filter_message().endpoint(BotMessages::message_handle))
        .branch(Update::filter_callback_query().endpoint(BotCallbacks::callback_handle))
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
