use crate::bot::utils as Utils;
use crate::database::models::Meme;
use crate::database::repository::MemeLikeRepository;
use crate::Application;
use std::error::Error;
use teloxide::prelude::*;
use teloxide::types::User;

pub async fn meme_of_week(
    bot: &Bot,
    app: &Application,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let (meme, likes) =
        MemeLikeRepository::new(app.database.clone()).get_top_meme(Utils::Period::Week)?;

    send_top(
        bot,
        &meme,
        likes,
        &String::from("Больше всех на этой неделе!"),
    )
    .await?;

    Ok(())
}

pub async fn meme_of_month(
    bot: &Bot,
    app: &Application,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let (meme, likes) =
        MemeLikeRepository::new(app.database.clone()).get_top_meme(Utils::Period::Month)?;

    send_top(
        bot,
        &meme,
        likes,
        &String::from("Больше всех в этом месяце!"),
    )
    .await?;

    Ok(())
}

pub async fn meme_of_year(
    bot: &Bot,
    app: &Application,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let (meme, likes) =
        MemeLikeRepository::new(app.database.clone()).get_top_meme(Utils::Period::Year)?;

    send_top(bot, &meme, likes, &String::from("Больше всех в этом году!")).await?;

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
    text: &String,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let user = get_chat_member(bot, meme).await?;

    bot.send_message(
        meme.chat_id(),
        format!(
            "{} твой мем набрал {} лайк(ов)!\n{}\nПоздравляю!",
            Utils::get_user_text(&user),
            likes,
            *text
        ),
    )
    .reply_to_message_id(meme.msg_id())
    .await
    .expect("Can't send 'top of' message");

    Ok(())
}
