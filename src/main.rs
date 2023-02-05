extern crate dotenv;
#[macro_use]
extern crate log;
extern crate pretty_env_logger;

use std::{env, sync::Arc};

use clap::Parser;
use dotenv::dotenv;
use teloxide::types::ParseMode;
use teloxide::{prelude::*, types::Me};

use bot::{
    callbacks::CallbackHandler,
    commands::{CommandsHandler, PrivateCommand, PublicCommand},
    messages::MessagesHandler,
};
use database::DBManager;
use utils::Period;

use crate::redis::RedisManager;

mod bot;
mod database;
mod redis;
mod utils;

pub struct Application {
    pub database: DBManager,
    pub redis: RedisManager,
    pub bot: Me,
    pub version: String,
}

#[rustfmt::skip]
#[derive(Debug, Parser)]
#[command(name = "tg_meme_bot", subcommand_required = true, arg_required_else_help = true)]
#[command(author = "Medic84", version, about = "Meme telegram bot for chats", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[rustfmt::skip]
#[derive(Parser, Debug)]
enum Commands {
    #[command(long_flag = "start", about = "Starts the bot daemon")]
    Start,
    #[command(long_flag = "meme_of_week", short_flag = 'w', about = "Send meme of week to chats")]
    MemeOfWeek,
    #[command(long_flag = "meme_of_month", short_flag = 'm', about = "Send meme of month to chats")]
    MemeOfMonth,
    #[command(long_flag = "meme_of_year", short_flag = 'y', about = "Send meme of year to chats")]
    MemeOfYear,
    #[command(long_flag = "users_update", about = "Update users of chats")]
    UpdateUsers,
    #[command(long_flag = "hash_update", about = "Update hashes of memes")]
    UpdateHashes,
}

#[tokio::main]
async fn main() {
    const VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");

    dotenv().ok();
    pretty_env_logger::init_timed();

    let args = Cli::parse();
    let bot = Bot::from_env().parse_mode(ParseMode::Html);
    let app = Arc::new(Application {
        database: DBManager::connect(&get_env("DATABASE_URL")),
        redis: RedisManager::connect(&get_env("REDIS_URL")),
        bot: bot.get_me().await.expect("Can't get bot information"),
        version: VERSION.unwrap_or("unknown").to_string(),
    });

    let chat_id_only: i64 = env::var("ONLY_FOR_CHAT_ID")
        .unwrap_or("0".to_string())
        .as_str()
        .parse::<i64>()
        .unwrap_or(0);

    if chat_id_only < 0 {
        app.redis.register_chat(chat_id_only);
    }

    match args.command {
        Commands::MemeOfWeek => {
            bot::top::send_top_stats(&bot, &app, Period::Week)
                .await
                .expect("Can't send meme of week");
        }
        Commands::MemeOfMonth => {
            bot::top::send_top_stats(&bot, &app, Period::Month)
                .await
                .expect("Can't send meme of month");
        }
        Commands::MemeOfYear => {
            bot::top::send_top_stats(&bot, &app, Period::Year)
                .await
                .expect("Can't send meme of year");
        }
        Commands::UpdateUsers => {
            if chat_id_only < 0 {
                utils::update_users(&bot, &app, chat_id_only)
                    .await
                    .expect("Can't update users");
            }
        }
        Commands::UpdateHashes => {
            utils::update_hashes(&bot, &app)
                .await
                .expect("Can't update hashes");
        }
        Commands::Start => {
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
                        .filter(move |m: Message| filter_messages(&m, &chat_id_only))
                        .filter_command::<PublicCommand>()
                        .endpoint(CommandsHandler::public_handle),
                )
                .branch(
                    Update::filter_message()
                        .filter(move |m: Message| filter_messages(&m, &chat_id_only))
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
    };
}

fn get_env(env: &'static str) -> String {
    env::var(env).unwrap_or_else(|_| panic!("{env} must be set"))
}

fn filter_messages(m: &Message, chat_id: &i64) -> bool {
    if *chat_id < 0 {
        (m.chat.is_group() || m.chat.is_supergroup()) && m.chat.id.0 == *chat_id
    } else {
        m.chat.is_group() || m.chat.is_supergroup()
    }
}
