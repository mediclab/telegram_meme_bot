extern crate dotenv;
#[macro_use]
extern crate log;
extern crate pretty_env_logger;

use std::sync::Arc;

use clap::Parser;
use dotenv::dotenv;

use crate::app::Application;
use crate::bot::top::Statistics;
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

    match args.command {
        Commands::MemeOfWeek => {
            let stats = Statistics::new(app.clone());
            stats.send(app.get_bot(), &Period::Week);
        }
        Commands::MemeOfMonth => {
            let stats = Statistics::new(app.clone());
            stats.send(app.get_bot(), &Period::Month);
        }
        Commands::MemeOfYear => {
            let stats = Statistics::new(app.clone());
            stats.send(app.get_bot(), &Period::Year);
        }
        Commands::Start => {
            info!("MemeBot version = {}", &app.config.app_version);
            info!("Starting scheduler...");

            let scheduler_handle = Scheduler::new(app.clone(), "16:05", "17:05", "18:05").init();

            info!("Starting dispatch...");

            app.bot_dispatch().await;

            info!("Dispatch stopped...");

            scheduler_handle.stop();

            info!("Scheduler stopped...");
        }
    };
}
