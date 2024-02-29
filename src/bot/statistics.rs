use crate::app::utils::Period;
use crate::app::{utils::Messages, Application};
use crate::database::models::{Meme, MemeLikeOperation};
use crate::nats::messages::StatisticMessage;
use std::sync::Arc;

pub struct Statistics {
    app: Arc<Application>,
}

impl Statistics {
    pub fn new(app: Arc<Application>) -> Self {
        Self { app }
    }

    pub fn send(&self, period: &Period) {
        match *period {
            Period::Week => {
                if Period::is_today_a_friday() {
                    info!("Send statistics of week");
                    self.send_by_period(period);
                } else {
                    debug!("Today is not a friday!");
                }
            }
            Period::Month => {
                if Period::is_today_a_last_month_day() {
                    info!("Send statistics of month");
                    self.send_by_period(period);
                } else {
                    debug!("Today is not a last month day!");
                }
            }
            Period::Year => {
                if Period::is_today_a_last_year_day() {
                    info!("Send statistics of year");
                    self.send_by_period(period);
                } else {
                    debug!("Today is not a last year day!");
                }
            }
        };
    }

    fn send_by_period(&self, period: &Period) {
        if let Some((meme, text)) = self.get_top_liked_meme(period) {
            let msg = StatisticMessage {
                chat_id: meme.chat_id,
                user_ids: vec![(String::from("{USERNAME}"), meme.user_id)],
                reply_id: Some(meme.msg_id.unwrap()),
                message: text,
            };
            self.app.nats.publish(&msg);
        } else {
            warn!("Can't get top liked mem for this period!");
        }

        let messages = vec![
            self.get_top_memesender(period),
            self.get_top_selfliker(period),
            self.get_top_liker(period),
            self.get_top_disliker(period),
        ]
        .into_iter()
        .filter(|i| i.is_some())
        .map(|i| i.unwrap_or_default())
        .collect::<Vec<((String, i64), String)>>();

        let message = messages.iter().map(|i| i.1.clone()).collect::<Vec<String>>();

        if !message.is_empty() {
            let msg = StatisticMessage {
                chat_id: self.app.config.bot.chat_id,
                user_ids: messages.into_iter().map(|i| i.0).collect::<Vec<(String, i64)>>(),
                message: format!("Хотели топов? Их есть у меня!\n\n{}", &message.join("\n\n")),
                reply_id: None,
            };
            self.app.nats.publish(&msg);
        } else {
            warn!("Can't get top statistics for this period!");
        }

        if let Some((meme, text)) = self.get_top_disliked_meme(period) {
            let msg = StatisticMessage {
                chat_id: meme.chat_id,
                user_ids: vec![(String::from("{USERNAME}"), meme.user_id)],
                reply_id: Some(meme.msg_id.unwrap()),
                message: text,
            };
            self.app.nats.publish(&msg);
        } else {
            warn!("Can't get top disliked mem for this period!");
        }
    }

    fn get_top_liked_meme(&self, period: &Period) -> Option<(Meme, String)> {
        match self.app.database.get_top_meme(period) {
            Ok((meme, likes)) => {
                let text = format!(
                    "{} твой мем набрал {}!\nБольше всех {}!\nПоздравляю! 🎉",
                    "{USERNAME}",
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

            let text = format!(
                "Вы только посмотрите, {} на твой мем наставили {}!\nТы точно уверен что делаешь все правильно? Может тебе больше не стоит заниматься юмором? 🤔",
                "{USERNAME}",
                Messages::pluralize(dislikes, ("дизлайк", "дизлайка", "дизлайков"))
            );

            return Some((meme, text));
        }

        None
    }

    fn get_top_memesender(&self, period: &Period) -> Option<((String, i64), String)> {
        if let Ok((user_id, count)) = self.app.database.get_top_memesender(period) {
            let placeholder = String::from("{MEMESENDER}");
            let period_text = Statistics::get_translations(period);

            let text = format!(
                "🤡 Мемомёт {}:\n{} отправил {} {}!",
                period_text.0,
                &placeholder,
                Messages::pluralize(count, ("мем", "мема", "мемов")),
                period_text.1
            );

            return Some(((placeholder, user_id), text));
        }

        None
    }

    fn get_top_selfliker(&self, period: &Period) -> Option<((String, i64), String)> {
        if let Ok((user_id, count)) = self.app.database.get_top_selflikes(period) {
            let placeholder = String::from("{SELFLIKER}");
            let period_text = Statistics::get_translations(period);

            if count > 4 {
                let text = format!(
                    "😈 Хитрец {}:\n{} лайкнул свои же мемы {} {}!",
                    period_text.0,
                    &placeholder,
                    Messages::pluralize(count, ("раз", "раза", "раз")),
                    period_text.1
                );

                return Some(((placeholder, user_id), text));
            }
        }

        None
    }

    fn get_top_liker(&self, period: &Period) -> Option<((String, i64), String)> {
        let query = self.app.database.get_top_likers(period, MemeLikeOperation::Like);

        if let Ok((user_id, count)) = query {
            let placeholder = String::from("{LIKER}");
            let period_text = Statistics::get_translations(period);

            let text = format!(
                "❤️ Добродеятель {}:\n{} поставил больше всех лайков {}!\nЦелых {}",
                period_text.0,
                &placeholder,
                period_text.1,
                Messages::pluralize(count, ("лайк", "лайка", "лайков")),
            );

            return Some(((placeholder, user_id), text));
        }

        None
    }

    fn get_top_disliker(&self, period: &Period) -> Option<((String, i64), String)> {
        let query = self.app.database.get_top_likers(period, MemeLikeOperation::Dislike);

        if let Ok((user_id, count)) = query {
            let placeholder = String::from("{DISLIKER}");
            let period_text = Statistics::get_translations(period);

            let text = format!(
                "😡 Засранец {}:\n{} поставил больше всех дизлайков {}!\nЦелых {}",
                period_text.0,
                &placeholder,
                period_text.1,
                Messages::pluralize(count, ("дизлайк", "дизлайка", "дизлайков")),
            );

            return Some(((placeholder, user_id), text));
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
