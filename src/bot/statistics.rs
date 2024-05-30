use crate::app::utils::Period;
use crate::app::{utils::Messages, Application};
use crate::database::entity::memes;
use crate::database::entity::prelude::{Memes, Users};
use crate::nats::messages::StatisticMessage;
use futures::executor::block_on;
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
                    block_on(self.send_by_period(period));
                } else {
                    debug!("Today is not a friday!");
                    block_on(self.send_by_period(period));
                }
            }
            Period::Month => {
                if Period::is_today_a_last_month_day() {
                    info!("Send statistics of month");
                    block_on(self.send_by_period(period));
                } else {
                    debug!("Today is not a last month day!");
                }
            }
            Period::Year => {
                if Period::is_today_a_last_year_day() {
                    info!("Send statistics of year");
                    block_on(self.send_by_period(period));
                } else {
                    debug!("Today is not a last year day!");
                }
            }
        };
    }

    async fn send_by_period(&self, period: &Period) {
        if let Some((meme, text)) = self.get_top_liked_meme(period).await {
            let msg = StatisticMessage {
                chat_id: meme.chat_id,
                user_ids: vec![(String::from("{USERNAME}"), meme.user_id)],
                reply_id: Some(meme.msg_id.unwrap()),
                message: text,
            };
            self.app.nats.publish(&msg).await;
        } else {
            warn!("Can't get top liked mem for this period!");
        }

        let res = vec![
            self.get_top_memesender(period).await,
            self.get_top_selfliker(period).await,
            self.get_top_liker(period).await,
            self.get_top_disliker(period).await,
        ];

        let messages = res.into_iter().flatten().collect::<Vec<((String, i64), String)>>();

        let message = messages.iter().map(|i| i.1.clone()).collect::<Vec<String>>();

        if !message.is_empty() {
            let msg = StatisticMessage {
                chat_id: self.app.config.bot.chat_id,
                user_ids: messages.into_iter().map(|i| i.0).collect::<Vec<(String, i64)>>(),
                message: format!("Хотели топов? Их есть у меня!\n\n{}", &message.join("\n\n")),
                reply_id: None,
            };
            self.app.nats.publish(&msg).await;
        } else {
            warn!("Can't get top statistics for this period!");
        }

        if *period == Period::Week {
            if let Some((meme, text)) = self.get_top_disliked_meme(period).await {
                let msg = StatisticMessage {
                    chat_id: meme.chat_id,
                    user_ids: vec![(String::from("{USERNAME}"), meme.user_id)],
                    reply_id: Some(meme.msg_id.unwrap()),
                    message: text,
                };
                self.app.nats.publish(&msg).await;
            } else {
                warn!("Can't get top disliked mem for this period!");
            }
        }
    }

    async fn get_top_liked_meme(&self, period: &Period) -> Option<(memes::Model, String)> {
        let (from, to) = period.dates();

        if let Some(meme) = Memes::get_max_liked(from, to).await {
            let like_counts = meme.count_all_likes().await?;
            let text = format!(
                "{} твой мем набрал {}!\nБольше всех {}!\nПоздравляю! 🎉",
                "{USERNAME}",
                Messages::pluralize(like_counts.likes, ("лайк", "лайка", "лайков")),
                Statistics::get_translations(period).1
            );

            return Some((meme, text));
        }

        None
    }

    async fn get_top_disliked_meme(&self, period: &Period) -> Option<(memes::Model, String)> {
        if *period != Period::Week {
            return None;
        }

        let (from, to) = period.dates();

        if let Some(meme) = Memes::get_max_disliked(from, to).await {
            let like_counts = meme.count_all_likes().await?;
            let text = format!(
                "Вы только посмотрите, {} на твой мем наставили {}!\nТы точно уверен что делаешь все правильно? Может тебе больше не стоит заниматься юмором? 🤔",
                "{USERNAME}",
                Messages::pluralize(like_counts.dislikes, ("дизлайк", "дизлайка", "дизлайков"))
            );

            return Some((meme, text));
        }

        None
    }

    async fn get_top_memesender(&self, period: &Period) -> Option<((String, i64), String)> {
        let (from, to) = period.dates();
        let res = Users::top_memesender(from, to).await;

        if let Some(top_user) = res {
            let placeholder = String::from("{MEMESENDER}");
            let period_text = Statistics::get_translations(period);

            let text = format!(
                "🤡 Мемомёт {}:\n{} отправил {} {}!",
                period_text.0,
                &placeholder,
                Messages::pluralize(top_user.count, ("мем", "мема", "мемов")),
                period_text.1
            );

            return Some(((placeholder, top_user.user_id), text));
        }

        None
    }

    async fn get_top_selfliker(&self, period: &Period) -> Option<((String, i64), String)> {
        let (from, to) = period.dates();
        let res = Users::top_selfliker(from, to).await;

        if let Some(top_user) = res {
            let placeholder = String::from("{SELFLIKER}");
            let period_text = Statistics::get_translations(period);

            if top_user.count > 4 {
                let text = format!(
                    "😈 Хитрец {}:\n{} лайкнул свои же мемы {} {}!",
                    period_text.0,
                    &placeholder,
                    Messages::pluralize(top_user.count, ("раз", "раза", "раз")),
                    period_text.1
                );

                return Some(((placeholder, top_user.user_id), text));
            }
        }

        None
    }

    async fn get_top_liker(&self, period: &Period) -> Option<((String, i64), String)> {
        let (from, to) = period.dates();
        let res = Users::top_liker(from, to).await;

        if let Some(top_user) = res {
            let placeholder = String::from("{LIKER}");
            let period_text = Statistics::get_translations(period);

            let text = format!(
                "❤️ Добродеятель {}:\n{} поставил больше всех лайков {}!\nЦелых {}",
                period_text.0,
                &placeholder,
                period_text.1,
                Messages::pluralize(top_user.count, ("лайк", "лайка", "лайков")),
            );

            return Some(((placeholder, top_user.user_id), text));
        }

        None
    }

    async fn get_top_disliker(&self, period: &Period) -> Option<((String, i64), String)> {
        let (from, to) = period.dates();
        let res = Users::top_disliker(from, to).await;

        if let Some(top_user) = res {
            let placeholder = String::from("{DISLIKER}");
            let period_text = Statistics::get_translations(period);

            let text = format!(
                "😡 Засранец {}:\n{} поставил больше всех дизлайков {}!\nЦелых {}",
                period_text.0,
                &placeholder,
                period_text.1,
                Messages::pluralize(top_user.count, ("дизлайк", "дизлайка", "дизлайков")),
            );

            return Some(((placeholder, top_user.user_id), text));
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
