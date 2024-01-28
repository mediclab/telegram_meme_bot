extern crate dotenv;
#[macro_use]
extern crate log;
extern crate pretty_env_logger;

use std::{env, sync::Arc};

use clap::Parser;
use dotenv::dotenv;
use teloxide::prelude::*;

use crate::app::Application;
use app::utils::Period;
use bot::{
    callbacks::CallbackHandler,
    commands::{CommandsHandler, PrivateCommand, PublicCommand},
    messages::MessagesHandler,
};

mod app;
mod bot;
mod database;
mod redis;

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
    dotenv().ok();
    pretty_env_logger::init_timed();

    let _guard = sentry::init(sentry::ClientOptions {
        release: sentry::release_name!(),
        ..Default::default()
    });

    let args = Cli::parse();
    let app = Arc::new(Application::new());

    let chat_id_only: i64 = env::var("ONLY_FOR_CHAT_ID")
        .unwrap_or("0".to_string())
        .as_str()
        .parse::<i64>()
        .unwrap_or(0);

    if chat_id_only < 0 {
        app.register_chat(chat_id_only).await;
    }

    match args.command {
        Commands::MemeOfWeek => {
            bot::top::send_top_stats(&app, Period::Week)
                .await
                .expect("Can't send meme of week");
        }
        Commands::MemeOfMonth => {
            bot::top::send_top_stats(&app, Period::Month)
                .await
                .expect("Can't send meme of month");
        }
        Commands::MemeOfYear => {
            bot::top::send_top_stats(&app, Period::Year)
                .await
                .expect("Can't send meme of year");
        }
        Commands::UpdateUsers => {
            if chat_id_only < 0 {
                app.update_users(chat_id_only)
                    .await
                    .expect("Can't update users");
            }
        }
        Commands::UpdateHashes => {
            app.update_hashes().await.expect("Can't update hashes");
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
                        .filter(|m: Message| m.chat.is_private())
                        .endpoint(MessagesHandler::private_handle),
                )
                .branch(
                    Update::filter_message()
                        .filter(move |m: Message| filter_messages(&m, &chat_id_only))
                        .endpoint(MessagesHandler::public_handle),
                )
                .branch(Update::filter_callback_query().endpoint(CallbackHandler::handle));

            info!("Starting dispatch...");

            Dispatcher::builder(app.bot.clone(), handler)
                .dependencies(dptree::deps![Arc::clone(&app)])
                .enable_ctrlc_handler()
                .build()
                .dispatch()
                .await;
        }
    };
}

fn filter_messages(m: &Message, chat_id: &i64) -> bool {
    if *chat_id < 0 {
        (m.chat.is_group() || m.chat.is_supergroup()) && m.chat.id.0 == *chat_id
    } else {
        m.chat.is_group() || m.chat.is_supergroup()
    }
}
