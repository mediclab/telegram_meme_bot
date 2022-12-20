extern crate dotenv;
#[macro_use] extern crate derive_builder;

mod db;
use db::DBManager;

mod handlers;

pub mod models;
pub mod schema;

use dotenv::dotenv;
use teloxide::prelude::*;
use clap::Parser;
use redis::Client as RedisClient;
use std::{sync::Arc, env};

pub struct BotState {
    pub db_manager: DBManager,
    pub redis: RedisClient
}

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
    
    let redis = redis::Client::open(
        env::var("REDIS_URL").expect("REDIS_URL must be set")
    ).unwrap();

    let db_manager: DBManager = DBManager::connect(
        env::var("DATABASE_URL").expect("DATABASE_URL must be set")
    );

    let state = Arc::new(
        BotState{ db_manager, redis }
    );

    let handler = 
        dptree::entry()
        .branch(Update::filter_message().endpoint(handlers::message_handle))
        .branch(Update::filter_callback_query().endpoint(handlers::callback_handle))
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
