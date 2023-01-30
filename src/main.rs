extern crate dotenv;
extern crate pretty_env_logger;
#[macro_use]
extern crate log;

mod bot;
mod database;
mod utils;

use bot::{
    callbacks::CallbackHandler,
    commands::{CommandsHandler, PrivateCommand, PublicCommand},
    messages::MessagesHandler,
};
use clap::{Arg, ArgMatches, Command};
use database::manager::DBManager;
use dotenv::dotenv;
use redis::Client as RedisClient;
use std::{env, process::exit, sync::Arc};
use teloxide::{prelude::*, types::Me};
use utils::Period;

pub struct Application {
    pub database: DBManager,
    pub redis: RedisClient,
    pub bot: Me,
    pub version: String,
}

#[tokio::main]
async fn main() {
    const VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");

    dotenv().ok();
    pretty_env_logger::init_timed();

    let bot = Bot::from_env();
    let app = Arc::new(Application {
        database: DBManager::connect(get_env("DATABASE_URL")),
        redis: RedisClient::open(get_env("REDIS_URL")).expect("Redis is not connected"),
        bot: bot.get_me().await.expect("Can't get bot information"),
        version: VERSION.unwrap_or("unknown").to_string(),
    });

    if is_arg("meme_of_week") {
        bot::top::send_top_stats(&bot, &app, Period::Week)
            .await
            .expect("Can't send meme of week");

        exit(0);
    }

    if is_arg("meme_of_month") {
        bot::top::send_top_stats(&bot, &app, Period::Month)
            .await
            .expect("Can't send meme of month");

        exit(0);
    }

    if is_arg("meme_of_year") {
        bot::top::send_top_stats(&bot, &app, Period::Year)
            .await
            .expect("Can't send meme of year");

        exit(0);
    }

    if is_arg("users_update") {
        // utils::update_users(&bot, &app, 0)
        //     .await
        //     .expect("Can't update users");

        exit(0);
    }

    if is_arg("hash_update") {
        utils::update_hashes(&bot, &app)
            .await
            .expect("Can't update hashes");

        exit(0);
    }

    if is_arg("start") {
        info!("MemeBot version = {}", &app.version);

        let handler = dptree::entry()
            .branch(
                Update::filter_message()
                    .filter(|m: Message| m.chat.is_private())
                    .filter_command::<PrivateCommand>()
                    .endpoint(CommandsHandler::private_handle),
            )
            .branch(
                Update::filter_message()
                    .filter(|m: Message| m.chat.is_group() || m.chat.is_supergroup())
                    .filter_command::<PublicCommand>()
                    .endpoint(CommandsHandler::public_handle),
            )
            .branch(
                Update::filter_message()
                    .filter(|m: Message| m.chat.is_group() || m.chat.is_supergroup())
                    .endpoint(MessagesHandler::handle),
            )
            .branch(Update::filter_callback_query().endpoint(CallbackHandler::handle));

        info!("Starting dispatch...");

        Dispatcher::builder(bot, handler)
            .dependencies(dptree::deps![Arc::clone(&app)])
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;
    }
}

fn cli() -> ArgMatches {
    Command::new("MemeBot")
        .arg(
            Arg::new("start")
                .long("start")
                .value_parser(["false", "true"])
                .default_value("false")
                .num_args(0)
                .default_missing_value("true")
                .help("Starts the bot daemon"),
        )
        .arg(
            Arg::new("meme_of_week")
                .long("meme_of_week")
                .short('w')
                .value_parser(["false", "true"])
                .default_value("false")
                .num_args(0)
                .default_missing_value("true")
                .help("Send meme of week to chats"),
        )
        .arg(
            Arg::new("meme_of_month")
                .long("meme_of_month")
                .short('m')
                .value_parser(["false", "true"])
                .default_value("false")
                .num_args(0)
                .default_missing_value("true")
                .help("Send meme of month to chats"),
        )
        .arg(
            Arg::new("meme_of_year")
                .long("meme_of_year")
                .short('y')
                .value_parser(["false", "true"])
                .default_value("false")
                .num_args(0)
                .default_missing_value("true")
                .help("Send meme of year to chats"),
        )
        .arg(
            Arg::new("users_update")
                .long("users_update")
                .value_parser(["false", "true"])
                .default_value("false")
                .num_args(0)
                .default_missing_value("true")
                .help("Update users of chats"),
        )
        .arg(
            Arg::new("hash_update")
                .long("hash_update")
                .value_parser(["false", "true"])
                .default_value("false")
                .num_args(0)
                .default_missing_value("true")
                .help("Update hashes of memes"),
        )
        .author("Medic84")
        .about("Meme telegram bot for chats")
        .get_matches()
}

fn is_arg(arg: &str) -> bool {
    cli().get_one::<String>(arg).unwrap().eq("true")
}

fn get_env(env: &str) -> String {
    env::var(env).unwrap_or_else(|_| panic!("{env} must be set"))
}
