use crate::app::utils::Period;
use crate::app::{utils as Utils, utils::Messages, Application};
use crate::bot::Bot as AppBot;
use crate::database::models::{Meme, MemeLikeOperation};
use futures::executor::block_on;
use std::sync::Arc;
use teloxide::prelude::*;

pub struct Statistics {
    app: Arc<Application>,
}

impl Statistics {
    pub fn new(app: Arc<Application>) -> Self {
        Self { app }
    }

    pub fn send(&self, bot: &AppBot, period: &Period) {
        match *period {
            Period::Week => {
                if Period::is_today_a_friday() {
                    self.send_by_period(bot, period);
                } else {
                    debug!("Today is not a friday!");
                }
            }
            Period::Month => {
                if Period::is_today_a_last_month_day() {
                    self.send_by_period(bot, period);
                } else {
                    debug!("Today is not a last month day!");
                }
            }
            Period::Year => {
                if Period::is_today_a_last_year_day() {
                    self.send_by_period(bot, period);
                } else {
                    debug!("Today is not a last year day!");
                }
            }
        };
    }

    fn send_by_period(&self, bot: &AppBot, period: &Period) {
        if let Some((meme, text)) = self.get_top_liked_meme(period) {
            block_on(
                bot.send_message(meme.chat_id(), text)
                    .reply_to_message_id(meme.msg_id())
                    .send(),
            )
            .expect("Can't send 'top of liked' message");
        } else {
            error!("Can't get top liked mem for this period!");
        }

        let message = vec![
            self.get_top_memesender(period),
            self.get_top_selfliker(period),
            self.get_top_liker(period),
            self.get_top_disliker(period),
        ]
        .into_iter()
        .filter(|i| i.is_some())
        .map(|i| i.unwrap_or_default())
        .collect::<Vec<String>>();

        if !message.is_empty() {
            block_on(
                bot.send_message(
                    ChatId(self.app.config.chat_id),
                    format!("Хотели топов? Их есть у меня!\n\n{}", &message.join("\n\n")),
                )
                .send(),
            )
            .expect("Can't send top statistics");
        } else {
            error!("Can't get top statistics for this period!");
        }

        if let Some((meme, text)) = self.get_top_disliked_meme(period) {
            block_on(
                bot.send_message(meme.chat_id(), text)
                    .reply_to_message_id(meme.msg_id())
                    .send(),
            )
            .expect("Can't send 'top of disliked' message");
        } else {
            info!("Can't get top disliked mem for this period!");
        }
    }

    fn get_top_liked_meme(&self, period: &Period) -> Option<(Meme, String)> {
        match self.app.database.get_top_meme(period) {
            Ok((meme, likes)) => {
                let user = self.app.get_chat_user(meme.chat_id, meme.user_id);

                let text = format!(
                    "{} твой мем набрал {}!\nБольше всех {}!\nПоздравляю! 🎉",
                    Utils::get_user_text(&user),
                    Messages::pluralize(likes, ("лайк", "лайка", "лайков")),
                    Statistics::get_translations(period).1
                );

                Some((meme, text))
            }
            Err(_) => {
                error!("Can't get top mem for this period!");
                None
            }
        }
    }

    fn get_top_disliked_meme(&self, period: &Period) -> Option<(Meme, String)> {
        if let Ok((meme, dislikes)) = self.app.database.get_max_disliked_meme(period) {
            if *period != Period::Week {
                return None;
            }

            let user = self.app.get_chat_user(meme.chat_id, meme.user_id);

            let text = format!(
                "Вы только посмотрите, {} на твой мем наставили {}!\nТы точно уверен что делаешь все правильно? Может тебе больше не стоит заниматься юмором? 🤔",
                Utils::get_user_text(&user),
                Messages::pluralize(dislikes, ("дизлайк", "дизлайка", "дизлайков"))
            );

            return Some((meme, text));
        }

        None
    }

    fn get_top_memesender(&self, period: &Period) -> Option<String> {
        if let Ok((user_id, count)) = self.app.database.get_top_memesender(period) {
            let user = self.app.get_chat_user(self.app.config.chat_id, user_id);
            let period_text = Statistics::get_translations(period);

            let text = format!(
                "🤡 Мемомёт {}:\n{} отправил {} {}!",
                period_text.0,
                Utils::get_user_text(&user),
                Messages::pluralize(count, ("мем", "мема", "мемов")),
                period_text.1
            );

            return Some(text);
        }

        None
    }

    fn get_top_selfliker(&self, period: &Period) -> Option<String> {
        if let Ok((user_id, count)) = self.app.database.get_top_selflikes(period) {
            let user = self.app.get_chat_user(self.app.config.chat_id, user_id);
            let period_text = Statistics::get_translations(period);

            if count > 4 {
                let text = format!(
                    "😈 Хитрец {}:\n{} лайкнул свои же мемы {} {}!",
                    period_text.0,
                    Utils::get_user_text(&user),
                    Messages::pluralize(count, ("раз", "раза", "раз")),
                    period_text.1
                );

                return Some(text);
            }
        }

        None
    }

    fn get_top_liker(&self, period: &Period) -> Option<String> {
        let query = self.app.database.get_top_likers(period, MemeLikeOperation::Like);

        if let Ok((user_id, count)) = query {
            let user = self.app.get_chat_user(self.app.config.chat_id, user_id);
            let period_text = Statistics::get_translations(period);

            let text = format!(
                "❤️ Добродеятель {}:\n{} поставил больше всех лайков {}!\nЦелых {}",
                period_text.0,
                Utils::get_user_text(&user),
                period_text.1,
                Messages::pluralize(count, ("лайк", "лайка", "лайков")),
            );

            return Some(text);
        }

        None
    }

    fn get_top_disliker(&self, period: &Period) -> Option<String> {
        let query = self.app.database.get_top_likers(period, MemeLikeOperation::Dislike);

        if let Ok((user_id, count)) = query {
            let user = self.app.get_chat_user(self.app.config.chat_id, user_id);
            let period_text = Statistics::get_translations(period);

            let text = format!(
                "😡 Засранец {}:\n{} поставил больше всех дизлайков {}!\nЦелых {}",
                period_text.0,
                Utils::get_user_text(&user),
                period_text.1,
                Messages::pluralize(count, ("лайк", "лайка", "лайков")),
            );

            return Some(text);
        }

        None
    }

    fn get_translations(period: &Period) -> (&str, &str) {
        match *period {
            Period::Week => ("недели", "на этой неделе"),
            Period::Month => ("месяца", "в этом месяце"),
            Period::Year => ("года", "в этом году"),
        }
    }
}
