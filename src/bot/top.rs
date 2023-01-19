use crate::bot::utils as Utils;
use crate::bot::utils::Period;
use crate::database::models::MemeLikeOperation;
use crate::database::repository::MemeLikeRepository;
use crate::Application;
use std::error::Error;
use teloxide::prelude::*;
use teloxide::types::User;

fn get_translations(period: &Period) -> (&str, &str) {
    match *period {
        Period::Week => ("недели", "на этой неделе"),
        Period::Month => ("месяца", "в этом месяце"),
        Period::Year => ("года", "в этом году"),
    }
}

pub async fn send_top_stats(
    bot: &Bot,
    app: &Application,
    period: Period,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut text: String;
    let repository = MemeLikeRepository::new(app.database.clone());
    let _res = repository.get_top_meme(&period);
    let trans = get_translations(&period);

    if _res.is_ok() {
        let (meme, likes) = _res.as_ref().unwrap();
        let user = get_chat_member(bot, meme.chat_id().0, meme.user_id().0).await?;

        text = format!(
            "{} твой мем набрал {likes} лайк(ов)!\nБольше всех {}!\nПоздравляю! {}",
            Utils::get_user_text(&user),
            trans.1,
            emojis::get_by_shortcode("tada").unwrap().as_str()
        );
    } else {
        println!("Can't get top mem for this period!");

        return Ok(());
    }

    let (meme, _) = _res.as_ref().unwrap();

    bot.send_message(meme.chat_id(), &text)
        .reply_to_message_id(meme.msg_id())
        .await
        .expect("Can't send 'top of' message");

    text = String::new();
    let _res = repository.get_top_memesender(&period);

    if _res.is_ok() {
        let (user_id, count) = _res.unwrap();
        let user = get_chat_member(bot, meme.chat_id().0, user_id as u64).await?;

        text = format!(
            "{} Мемомёт {}:\n{} отправил {count} мемов {}!\n\n",
            emojis::get_by_shortcode("clown_face").unwrap().as_str(),
            trans.0,
            Utils::get_user_text(&user),
            trans.1
        );
    }

    let _res = repository.get_top_selflikes(&period);

    if _res.is_ok() {
        let (user_id, count) = _res.unwrap();
        let user = get_chat_member(bot, meme.chat_id().0, user_id as u64).await?;

        text = format!(
            "{text}{} Хитрец {}:\n{} лайкнул свои же мемы {count} раз {}!\n\n",
            emojis::get_by_shortcode("smiling_imp").unwrap().as_str(),
            trans.0,
            Utils::get_user_text(&user),
            trans.1
        );
    }

    let _res = repository.get_top_likers(&period, MemeLikeOperation::Like);

    if _res.is_ok() {
        let (user_id, count) = _res.unwrap();
        let user = get_chat_member(bot, meme.chat_id().0, user_id as u64).await?;

        text = format!(
            "{text}{} Добродеятель {}:\n{} поставил больше всех лайков {}!\nЦелых {count} лайков\n\n",
            emojis::get_by_shortcode("heart").unwrap().as_str(),
            trans.0,
            Utils::get_user_text(&user),
            trans.1
        );
    }

    let _res = repository.get_top_likers(&period, MemeLikeOperation::Dislike);

    if _res.is_ok() {
        let (user_id, count) = _res.unwrap();
        let user = get_chat_member(bot, meme.chat_id().0, user_id as u64).await?;

        text = format!(
            "{text}{} Засранец {}:\n{} поставил больше всех дизлайков {}!\nЦелых {count} дизлайков",
            emojis::get_by_shortcode("rage").unwrap().as_str(),
            trans.0,
            Utils::get_user_text(&user),
            trans.1
        );
    }

    if !text.is_empty() {
        bot.send_message(
            meme.chat_id(),
            format!("Хотели топов? Их есть у меня!\n\n{}", &text),
        )
        .await
        .expect("Can't send 'top of' message");
    }

    Ok(())
}

async fn get_chat_member(
    bot: &Bot,
    chat_id: i64,
    user_id: u64,
) -> Result<User, Box<dyn Error + Send + Sync>> {
    Ok(bot
        .get_chat_member(ChatId(chat_id), UserId(user_id))
        .await
        .expect("Can't get chat member")
        .user)
}
