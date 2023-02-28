use anyhow::Result;
use teloxide::prelude::*;

use crate::app::utils as Utils;
use crate::app::utils::Messages;
use crate::app::Application;
use crate::database::models::MemeLikeOperation;

pub async fn send_top_stats(app: &Application, period: Utils::Period) -> Result<()> {
    let mut text: String;
    let _res = app.database.get_top_meme(&period);
    let period_text = get_translations(&period);

    if _res.is_ok() {
        let (meme, likes) = _res.as_ref().unwrap();
        let user = app
            .get_chat_user(meme.chat_id().0, meme.user_id().0)
            .await?;

        text = format!(
            "{} твой мем набрал {}!\nБольше всех {}!\nПоздравляю! {}",
            Utils::get_user_text(&user),
            Messages::pluralize(*likes, ("лайк", "лайка", "лайков")),
            period_text.1,
            emojis::get_by_shortcode("tada").unwrap().as_str()
        );
    } else {
        error!("Can't get top mem for this period!");

        return Ok(());
    }

    let (meme, _) = _res.as_ref().unwrap();

    app.bot
        .send_message(meme.chat_id(), &text)
        .reply_to_message_id(meme.msg_id())
        .await
        .expect("Can't send 'top of' message");

    text = String::new();
    let _res = app.database.get_top_memesender(&period);

    if _res.is_ok() {
        let (user_id, count) = _res.unwrap();
        let user = app.get_chat_user(meme.chat_id().0, user_id as u64).await?;

        text = format!(
            "{} Мемомёт {}:\n{} отправил {} {}!\n\n",
            emojis::get_by_shortcode("clown_face").unwrap().as_str(),
            period_text.0,
            Utils::get_user_text(&user),
            Messages::pluralize(count, ("мем", "мема", "мемов")),
            period_text.1
        );
    }

    let _res = app.database.get_top_selflikes(&period);

    if _res.is_ok() {
        let (user_id, count) = _res.unwrap();
        let user = app.get_chat_user(meme.chat_id().0, user_id as u64).await?;

        text = format!(
            "{text}{} Хитрец {}:\n{} лайкнул свои же мемы {} {}!\n\n",
            emojis::get_by_shortcode("smiling_imp").unwrap().as_str(),
            period_text.0,
            Utils::get_user_text(&user),
            Messages::pluralize(count, ("раз", "раза", "раз")),
            period_text.1
        );
    }

    let _res = app
        .database
        .get_top_likers(&period, MemeLikeOperation::Like);

    if _res.is_ok() {
        let (user_id, count) = _res.unwrap();
        let user = app.get_chat_user(meme.chat_id().0, user_id as u64).await?;

        text = format!(
            "{text}{} Добродеятель {}:\n{} поставил больше всех лайков {}!\nЦелых {}\n\n",
            emojis::get_by_shortcode("heart").unwrap().as_str(),
            period_text.0,
            Utils::get_user_text(&user),
            period_text.1,
            Messages::pluralize(count, ("лайк", "лайка", "лайков")),
        );
    }

    let _res = app
        .database
        .get_top_likers(&period, MemeLikeOperation::Dislike);

    if _res.is_ok() {
        let (user_id, count) = _res.unwrap();
        let user = app.get_chat_user(meme.chat_id().0, user_id as u64).await?;

        text = format!(
            "{text}{} Засранец {}:\n{} поставил больше всех дизлайков {}!\nЦелых {}",
            emojis::get_by_shortcode("rage").unwrap().as_str(),
            period_text.0,
            Utils::get_user_text(&user),
            period_text.1,
            Messages::pluralize(count, ("дизлайк", "дизлайка", "дизлайков")),
        );
    }

    if !text.is_empty() {
        app.bot
            .send_message(
                meme.chat_id(),
                format!("Хотели топов? Их есть у меня!\n\n{}", &text),
            )
            .await
            .expect("Can't send 'top of' message");
    }

    let _res = app.database.get_max_disliked_meme(&period);

    if _res.is_ok() && period == Utils::Period::Week {
        let (meme, dislikes) = _res.as_ref().unwrap();
        let user = app
            .get_chat_user(meme.chat_id().0, meme.user_id().0)
            .await?;

        text = format!(
            "Вы только посмотрите, {} на твой мем наставили {}!\n. Ты точно уверен что делаешь все правильно? Может тебе больше не стоит заниматься юмором? {}",
            Utils::get_user_text(&user),
            Messages::pluralize(*dislikes, ("дизлайк", "дизлайка", "дизлайков")),
            emojis::get_by_shortcode("thinking").unwrap().as_str()
        );

        app.bot
            .send_message(meme.chat_id(), &text)
            .reply_to_message_id(meme.msg_id())
            .await
            .expect("Can't send 'top of' message");
    }

    Ok(())
}

fn get_translations(period: &Utils::Period) -> (&str, &str) {
    match *period {
        Utils::Period::Week => ("недели", "на этой неделе"),
        Utils::Period::Month => ("месяца", "в этом месяце"),
        Utils::Period::Year => ("года", "в этом году"),
    }
}
