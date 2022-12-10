extern crate dotenv;

use dotenv::dotenv;
use teloxide::{prelude::*, types::{InputFile, ReplyMarkup, InlineKeyboardButton}};
use clap::Parser;

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

    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        match msg.photo() {
            Some(photo) => {
                bot.delete_message(msg.chat.id, msg.id).await?;
                bot.send_photo(msg.chat.id, InputFile::file_id(&photo[0].file.id))
                .reply_markup(ReplyMarkup::inline_kb(vec![vec![
                    InlineKeyboardButton::callback(emojis::get_by_shortcode('heart').unwrap(), String::from("Like")),
                    InlineKeyboardButton::callback(String::from("Dislike"), String::from("Dislike"))
                ]]))
                .await?;
            },
            None => {}
        }
        
        Ok(())
    })
    .await;
}
