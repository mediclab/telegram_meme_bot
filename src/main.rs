extern crate dotenv;

mod bot;
mod database;

use dotenv::dotenv;
use teloxide::{prelude::*, types::Me};
use std::{sync::Arc, process::exit, env};
use clap::{ArgMatches, Command, Arg};
use database::{
    manager::DBManager,
    repository::MemeLikeRepository,
};
use bot::{
    messages::MessagesHandler,
    callbacks::CallbackHandler,
    commands::{CommandsHandler, Command as BotCommands}
};

pub struct Application {
    pub database: DBManager,
    pub bot: Me,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    pretty_env_logger::init();

    let bot = Bot::from_env();
    let state = Arc::new(
        Application {
            database: DBManager::connect(
                env::var("DATABASE_URL").expect("DATABASE_URL must be set")
            ),
            bot: bot.get_me().await.expect("Can't get bot information"),
        }
    );

    if is_arg("meme_of_week") {
        let meme = MemeLikeRepository::new(state.database.clone())
            .meme_of_week()
            .unwrap()
        ;

        let user = bot.get_chat_member(
            meme.0.chat_id(),
            meme.0.user_id()
        ).await.expect("Can't get user for meme of week").user;

        bot.send_message(
            meme.0.chat_id(),
            format!(
                    "{} твой мем набрал {} лайк(ов)!\nБольше всех на этой неделе!\nПоздравляю!",
                    bot::utils::get_user_text(&user),
                    meme.1
            )
        ).reply_to_message_id(meme.0.msg_id()).await.expect("Can't send message");

        exit(0);
    }

    if is_arg("meme_of_month") {
        let meme = MemeLikeRepository::new(state.database.clone())
            .meme_of_month()
            .unwrap()
        ;

        let user = bot.get_chat_member(
            meme.0.chat_id(),
            meme.0.user_id()
        ).await.expect("Can't get user for meme of month").user;

        bot.send_message(
            meme.0.chat_id(),
            format!(
                "{} твой мем набрал {} лайк(ов)!\nБольше всех в этом месяце!\nПоздравляю!",
                bot::utils::get_user_text(&user),
                meme.1
            )
        ).reply_to_message_id(meme.0.msg_id()).await.expect("Can't send message");

        exit(0);
    }

    if is_arg("meme_of_year") {
        let meme = MemeLikeRepository::new(state.database.clone())
            .meme_of_year()
            .unwrap()
        ;

        let user = bot.get_chat_member(
            meme.0.chat_id(),
            meme.0.user_id()
        ).await.expect("Can't get user for meme of year").user;

        bot.send_message(
            meme.0.chat_id(),
            format!(
                "{} твой мем набрал {} лайк(ов)!\nБольше всех в этом году!\nПоздравляю!",
                bot::utils::get_user_text(&user),
                meme.1
            )
        ).reply_to_message_id(meme.0.msg_id()).await.expect("Can't send message");

        exit(0);
    }

    if is_arg("start") {
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

        .arg(Arg::new("meme_of_week")
            .long("meme_of_week")
            .short('w')
            .value_parser(["false", "true"])
            .default_value("false")
            .num_args(0)
            .default_missing_value("true")
            .help("Send meme of week to chats")
        )

        .arg(Arg::new("meme_of_month")
            .long("meme_of_month")
            .short('m')
            .value_parser(["false", "true"])
            .default_value("false")
            .num_args(0)
            .default_missing_value("true")
            .help("Send meme of month to chats")
        )

        .arg(Arg::new("meme_of_year")
            .long("meme_of_year")
            .short('y')
            .value_parser(["false", "true"])
            .default_value("false")
            .num_args(0)
            .default_missing_value("true")
            .help("Send meme of year to chats")
        )

        .author("Medic84")
        .about("Meme telegram bot for chats")
        .get_matches()
}

fn is_arg(arg: &str) -> bool {
    cli().get_one::<String>(arg).unwrap().eq("true")
}
