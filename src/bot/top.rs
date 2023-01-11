use crate::bot::utils as Utils;
use crate::database::models::Meme;
use crate::database::repository::MemeLikeRepository;
use crate::Application;
use std::error::Error;
use teloxide::prelude::*;
use teloxide::types::User;

pub enum Top {
    Week,
    Month,
    Year,
}

impl Top {
    pub fn name(&self) -> String {
        match *self {
            Top::Week => String::from("Больше всех на этой неделе!"),
            Top::Month => String::from("Больше всех в этом месяце!"),
            Top::Year => String::from("Больше всех в этом году!"),
        }
    }
}

pub async fn meme_of_week(
    bot: &Bot,
    app: &Application,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let (meme, likes) = MemeLikeRepository::new(app.database.clone())
        .meme_of_week()
        .unwrap();

    send_top(bot, &meme, likes, Top::Week).await?;

    Ok(())
}

pub async fn meme_of_month(
    bot: &Bot,
    app: &Application,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let (meme, likes) = MemeLikeRepository::new(app.database.clone())
        .meme_of_month()
        .unwrap();

    send_top(bot, &meme, likes, Top::Month).await?;

    Ok(())
}

pub async fn meme_of_year(
    bot: &Bot,
    app: &Application,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let (meme, likes) = MemeLikeRepository::new(app.database.clone())
        .meme_of_year()
        .unwrap();

    send_top(bot, &meme, likes, Top::Year).await?;

    Ok(())
}

async fn get_chat_member(bot: &Bot, meme: &Meme) -> Result<User, Box<dyn Error + Send + Sync>> {
    Ok(bot
        .get_chat_member(meme.chat_id(), meme.user_id())
        .await
        .expect("Can't get chat member")
        .user)
}

async fn send_top(
    bot: &Bot,
    meme: &Meme,
    likes: i64,
    text: Top,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let user = get_chat_member(bot, meme).await?;

    bot.send_message(
        meme.chat_id(),
        format!(
            "{} твой мем набрал {} лайк(ов)!\n{}\nПоздравляю!",
            Utils::get_user_text(&user),
            likes,
            text.name()
        ),
    )
    .reply_to_message_id(meme.msg_id())
    .await
    .expect("Can't send 'top of' message");

    Ok(())
}
