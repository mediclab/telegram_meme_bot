use crate::app::{utils as Utils, utils::Messages, Application};
use crate::database::models::MemeLikeOperation;
use anyhow::Result;
use teloxide::prelude::*;

pub async fn send_top_stats(app: &Application, period: Utils::Period) -> Result<()> {
    let mut text: String;
    let period_text = get_translations(&period);
    let chat_id: i64;

    debug!(
        "Selected period from: {} to: {}",
        period.dates().0,
        period.dates().1
    );

    match period {
        Utils::Period::Week => {
            if !Utils::Period::is_today_a_friday() {
                debug!("Today is not a Friday!");
                return Ok(());
            }
        }
        Utils::Period::Month => {
            if !Utils::Period::is_today_a_last_month_day() {
                debug!("Today is not a Last Month Day!");
                return Ok(());
            }
        }
        Utils::Period::Year => {
            if !Utils::Period::is_today_a_last_year_day() {
                debug!("Today is not a Last Year Day!");
                return Ok(());
            }
        }
    }

    match app.database.get_top_meme(&period) {
        Ok((meme, likes)) => {
            let user = app.get_chat_user(meme.chat_id, meme.user_id as u64).await?;

            text = format!(
                "{} твой мем набрал {}!\nБольше всех {}!\nПоздравляю! 🎉",
                Utils::get_user_text(&user),
                Messages::pluralize(likes, ("лайк", "лайка", "лайков")),
                period_text.1
            );

            app.bot
                .send_message(meme.chat_id(), &text)
                .reply_to_message_id(meme.msg_id())
                .await
                .expect("Can't send 'top of' message");

            chat_id = meme.chat_id().0;
        }
        Err(_) => {
            error!("Can't get top mem for this period!");

            return Ok(());
        }
    }

    text = String::new();

    if let Ok((user_id, count)) = app.database.get_top_memesender(&period) {
        let user = app.get_chat_user(chat_id, user_id as u64).await?;

        text = format!(
            "🤡 Мемомёт {}:\n{} отправил {} {}!\n\n",
            period_text.0,
            Utils::get_user_text(&user),
            Messages::pluralize(count, ("мем", "мема", "мемов")),
            period_text.1
        );
    };

    if let Ok((user_id, count)) = app.database.get_top_selflikes(&period) {
        let user = app.get_chat_user(chat_id, user_id as u64).await?;

        if count > 4 {
            text = format!(
                "{text}😈 Хитрец {}:\n{} лайкнул свои же мемы {} {}!\n\n",
                period_text.0,
                Utils::get_user_text(&user),
                Messages::pluralize(count, ("раз", "раза", "раз")),
                period_text.1
            );
        }
    }

    if let Ok((user_id, count)) = app
        .database
        .get_top_likers(&period, MemeLikeOperation::Like)
    {
        let user = app.get_chat_user(chat_id, user_id as u64).await?;

        text = format!(
            "{text}❤️ Добродеятель {}:\n{} поставил больше всех лайков {}!\nЦелых {}\n\n",
            period_text.0,
            Utils::get_user_text(&user),
            period_text.1,
            Messages::pluralize(count, ("лайк", "лайка", "лайков")),
        );
    }

    if let Ok((user_id, count)) = app
        .database
        .get_top_likers(&period, MemeLikeOperation::Dislike)
    {
        let user = app.get_chat_user(chat_id, user_id as u64).await?;

        text = format!(
            "{text}😡 Засранец {}:\n{} поставил больше всех дизлайков {}!\nЦелых {}",
            period_text.0,
            Utils::get_user_text(&user),
            period_text.1,
            Messages::pluralize(count, ("дизлайк", "дизлайка", "дизлайков")),
        );
    }

    if !text.is_empty() {
        app.bot
            .send_message(
                ChatId(chat_id),
                format!("Хотели топов? Их есть у меня!\n\n{}", &text),
            )
            .await
            .expect("Can't send 'top of' message");
    }

    if let Ok((meme, dislikes)) = app.database.get_max_disliked_meme(&period) {
        if period != Utils::Period::Week {
            return Ok(());
        }

        let user = app.get_chat_user(meme.chat_id, meme.user_id as u64).await?;

        text = format!(
            "Вы только посмотрите, {} на твой мем наставили {}!\nТы точно уверен что делаешь все правильно? Может тебе больше не стоит заниматься юмором? 🤔",
            Utils::get_user_text(&user),
            Messages::pluralize(dislikes, ("дизлайк", "дизлайка", "дизлайков"))
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
