extern crate dotenv;
#[macro_use]
extern crate log;
extern crate pretty_env_logger;

use chrono::NaiveDateTime;
use std::sync::Arc;

use clap::Parser;
use dotenv::dotenv;
use teloxide::{
    dispatching::dialogue::{serializer::Json, RedisStorage},
    dptree,
};

use crate::app::Application;
use crate::bot::{statistics::Statistics, BotManager};
use crate::database::Database;
use crate::redis::RedisManager;
use crate::scheduler::Scheduler;
use app::utils::Period;

mod app;
mod bot;
mod database;
mod redis;
mod scheduler;

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
    #[command(long_flag = "meme_of_custom", short_flag = 'c', about = "Send meme of custom period to chats")]
    MemeOfCustom {
        #[arg(required = true, help = "Set date and time start period")]
        from: NaiveDateTime,
        #[arg(required = true, help = "Set date and time end period")]
        to: NaiveDateTime
    },
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
    let scheduler = Scheduler::new();

    let db = Database::new(&app.config.db_url).await;
    db.migrate().await.expect("Can't migrate database");
    let redis = RedisManager::connect(&app.config.redis_url);
    let bot = BotManager::new(&app.config.bot);

    database::INSTANCE.set(db).expect("Can't set database");
    bot::INSTANCE.set(bot).expect("Can't set BotManager");
    redis::INSTANCE.set(redis).expect("Can't set RedisManager");

    app.register_chat();
    app.check_version();

    match args.command {
        Commands::MemeOfWeek => {
            let stats = Statistics::new();
            stats.send(&Period::Week).await;
        }
        Commands::MemeOfMonth => {
            let stats = Statistics::new();
            stats.send(&Period::Month).await;
        }
        Commands::MemeOfYear => {
            let stats = Statistics::new();
            stats.send(&Period::Year).await;
        }
        Commands::MemeOfCustom { from, to } => {
            let stats = Statistics::new();
            stats
                .send(&Period::Custom {
                    from: from.and_utc(),
                    to: to.and_utc(),
                })
                .await;
        }
        Commands::Start => {
            info!("MemeBot version = {}", &app.config.app_version);

            info!("Starting scheduler...");
            scheduler.handle().await.expect("Can't run scheduler");

            info!("Starting dispatch...");
            BotManager::global()
                .dispatch(dptree::deps![
                    app.clone(),
                    RedisStorage::open(&app.config.redis_url.clone(), Json)
                        .await
                        .expect("Can't connect dialogues on redis")
                ])
                .await;

            info!("Shutdown bot...");
        }
    };
}
