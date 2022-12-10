use teloxide::{prelude::*, types::{InputFile, ReplyMarkup, InlineKeyboardButton, InlineKeyboardMarkup}};
use std::error::Error;
use redis::{Client as RedisClient, RedisError};
use redis::{aio::Connection, AsyncCommands, FromRedisValue};

pub async fn message_handle(bot: Bot, msg: Message, client: RedisClient) -> Result<(), Box<dyn Error + Send + Sync>> {
    let user = msg.from().unwrap();
    let mut redis = client.get_async_connection().await?;

    let user_text = match &user.username {
        Some(uname) => format!("@{}", uname),
        None => format!("[{}](tg://user?id={})", user.first_name, user.id)
    };

    match msg.photo() {
        Some(photo) => {
            redis.set(format!("likes_{:?}", msg.id), 0).await?;

            bot.delete_message(msg.chat.id, msg.id).await?;
            bot.send_photo(msg.chat.id, InputFile::file_id(&photo[0].file.id))
            .caption(format!("Оцените мем {}", user_text))
            .reply_markup(ReplyMarkup::inline_kb(vec![vec![
                InlineKeyboardButton::callback(
                    format!("{} Like ({})", String::from(emojis::get_by_shortcode("heart").unwrap().as_str()), 0),
                    String::from("Like")
                ),
                InlineKeyboardButton::callback(
                    format!("{} Dislike ({})",String::from(emojis::get_by_shortcode("broken_heart").unwrap().as_str()), 0),
                    String::from("Dislike")
                )
            ]]))
            .await?;
        },
        None => {}
    }
    
    Ok(())
}

pub async fn callback_handle(bot: Bot, callback: CallbackQuery, client: RedisClient) -> Result<(), Box<dyn Error + Send + Sync>> {
    let msg = callback.message.unwrap();
    let mut redis = client.get_async_connection().await?;

    match callback.data.unwrap().as_str() {
        "Like" => {
            let likes = redis.get(format!("likes_{:?}", msg.id)).await.map_err(|err| {});
            println!("{:?}", FromRedisValue::from_redis_value(&likes).map_err(|e| RedisError(e).into()));

            let request = bot.edit_message_reply_markup(format!("@{}", msg.chat.username().unwrap()), msg.id)
            .reply_markup(
                InlineKeyboardMarkup::new(
                    vec![vec![
                        InlineKeyboardButton::callback(
                            format!(
                                "{} Like (0)", String::from(emojis::get_by_shortcode("heart").unwrap().as_str()),
                                //FromRedisValue::from_redis_value(&likes).map_err(|err| {})
                            ),
                            String::from("Like")
                        ),
                        InlineKeyboardButton::callback(
                            format!("{} Dislike ({})",String::from(emojis::get_by_shortcode("broken_heart").unwrap().as_str()), 0),
                            String::from("Dislike")
                        )
                    ]]
                )  
            );

            request.send();
        },
        "Dislike" => {

        }
        _ => {},
    }
    Ok(())
}