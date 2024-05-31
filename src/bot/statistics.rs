use crate::app::utils::{get_user_text, Messages, Period};
use crate::bot::BotManager;
use crate::database::entity::prelude::{Memes, Users};
use futures::future::join_all;
use futures::FutureExt;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::ChatId;
use teloxide::requests::Requester;
use teloxide::types::MessageId;

#[derive(Debug, Clone)]
pub struct Message {
    text: String,
    placeholder: String,
    user_id: i64,
    separate: bool,
    reply_id: Option<i64>,
}

pub struct Statistics {
    bot: BotManager,
}

impl Statistics {
    pub fn new() -> Self {
        let bot = BotManager::global().clone();

        Self { bot }
    }

    pub async fn send(&self, period: &Period) {
        match *period {
            Period::Week => {
                if Period::is_today_a_friday() {
                    info!("Send statistics of week");
                    self.send_by_period(period).await;
                } else {
                    debug!("Today is not a friday!");
                }
            }
            Period::Month => {
                if Period::is_today_a_last_month_day() {
                    info!("Send statistics of month");
                    self.send_by_period(period).await;
                } else {
                    debug!("Today is not a last month day!");
                }
            }
            Period::Year => {
                if Period::is_today_a_last_year_day() {
                    info!("Send statistics of year");
                    self.send_by_period(period).await;
                } else {
                    debug!("Today is not a last year day!");
                }
            }
        };
    }

    async fn send_by_period(&self, period: &Period) {
        let res = join_all(vec![
            self.get_top_liked_meme(period).boxed(),
            self.get_top_memesender(period).boxed(),
            self.get_top_selfliker(period).boxed(),
            self.get_top_liker(period).boxed(),
            self.get_top_disliker(period).boxed(),
            self.get_top_disliked_meme(period).boxed(),
        ])
        .await;

        let messages = res.into_iter().flatten().collect::<Vec<Message>>();

        let bot = &self.bot;
        let chat_id = self.bot.chat_id;

        let mut buffer: Vec<String> = Vec::new();

        for message in messages {
            let user = self.bot.get_chat_user(message.user_id).await;
            let text = message.text.replace(&message.placeholder, &get_user_text(&user));

            if message.separate {
                if !buffer.is_empty() {
                    bot.get()
                        .send_message(
                            ChatId(chat_id),
                            &format!("Хотели топов? Их есть у меня!\n\n{}", &buffer.join("\n\n")),
                        )
                        .await
                        .expect("Can't send message");
                    buffer.clear();
                }

                let mut s = bot.get().send_message(ChatId(chat_id), &text);

                if let Some(reply_id) = message.reply_id {
                    s = s.reply_to_message_id(MessageId(reply_id as i32));
                }

                s.await.expect("Can't send message");
            } else {
                buffer.push(text);
            }
        }

        if !buffer.is_empty() {
            bot.get()
                .send_message(
                    ChatId(chat_id),
                    &format!("Хотели топов? Их есть у меня!\n\n{}", &buffer.join("\n\n")),
                )
                .await
                .expect("Can't send message");
            buffer.clear();
        }
    }

    async fn get_top_liked_meme(&self, period: &Period) -> Option<Message> {
        let (from, to) = period.dates();

        if let Some(meme) = Memes::get_max_liked(from, to).await {
            let placeholder = String::from("{USERNAME}");
            let like_counts = meme.count_all_likes().await?;
            let text = format!(
                "{} твой мем набрал {}!\nБольше всех {}!\nПоздравляю! 🎉",
                &placeholder,
                Messages::pluralize(like_counts.likes, ("лайк", "лайка", "лайков")),
                Statistics::get_translations(period).1
            );

            return Some(Message {
                text,
                placeholder,
                user_id: meme.user_id,
                separate: true,
                reply_id: meme.msg_id,
            });
        }

        None
    }

    async fn get_top_disliked_meme(&self, period: &Period) -> Option<Message> {
        if *period != Period::Week {
            return None;
        }

        let (from, to) = period.dates();

        if let Some(meme) = Memes::get_max_disliked(from, to).await {
            let placeholder = String::from("{USERNAME}");
            let like_counts = meme.count_all_likes().await?;
            let text = format!(
                "Вы только посмотрите, {} на твой мем наставили {}!\nТы точно уверен что делаешь все правильно? Может тебе больше не стоит заниматься юмором? 🤔",
                &placeholder,
                Messages::pluralize(like_counts.dislikes, ("дизлайк", "дизлайка", "дизлайков"))
            );

            return Some(Message {
                text,
                placeholder,
                user_id: meme.user_id,
                separate: true,
                reply_id: meme.msg_id,
            });
        }

        None
    }

    async fn get_top_memesender(&self, period: &Period) -> Option<Message> {
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

            return Some(Message {
                text,
                placeholder,
                user_id: top_user.user_id,
                separate: false,
                reply_id: None,
            });
        }

        None
    }

    async fn get_top_selfliker(&self, period: &Period) -> Option<Message> {
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

                return Some(Message {
                    text,
                    placeholder,
                    user_id: top_user.user_id,
                    separate: false,
                    reply_id: None,
                });
            }
        }

        None
    }

    async fn get_top_liker(&self, period: &Period) -> Option<Message> {
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

            return Some(Message {
                text,
                placeholder,
                user_id: top_user.user_id,
                separate: false,
                reply_id: None,
            });
        }

        None
    }

    async fn get_top_disliker(&self, period: &Period) -> Option<Message> {
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

            return Some(Message {
                text,
                placeholder,
                user_id: top_user.user_id,
                separate: false,
                reply_id: None,
            });
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
