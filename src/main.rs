extern crate dotenv;

mod handlers;
mod db_connection;
pub mod models;
pub mod schema;

use dotenv::dotenv;
use teloxide::prelude::*;
use clap::Parser;

use redis::Client as RedisClient;
use db_connection::PgPool;
use std::{sync::Arc, env};

pub struct BotState {
    pub db_pool: PgPool,
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
    let db_pool: PgPool = db_connection::establish_connection();
    let state = Arc::new(BotState{ db_pool, redis });

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
