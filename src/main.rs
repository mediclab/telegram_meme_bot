extern crate dotenv;

use dotenv::dotenv;
use teloxide::prelude::*;
use clap::Parser;

pub mod handlers;

#[derive(Parser,Default,Debug)]
#[clap(author="Medic84", version, about="Meme telegram bot for chats")]
struct Arguments {
    /// Run the bot daemon
    bot: bool,
    max_depth: usize,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    pretty_env_logger::init();

    //let args = Arguments::parse();

    let bot = Bot::from_env();
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();

    let handler = 
        dptree::entry()
        .branch(Update::filter_message().endpoint(handlers::message_handle))
        .branch(Update::filter_callback_query().endpoint(handlers::callback_handle))
    ;

    println!("Starting dispatch...");

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![client])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await
    ;
}
