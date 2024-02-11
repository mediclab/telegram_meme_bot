use crate::app::utils::get_user_text;
use crate::bot;
use crate::bot::Bot;
use crate::nats::messages::StatisticMessage;
use async_nats::Client;
use futures::executor::block_on;
use futures::StreamExt;
use serde_json::json;
use std::str;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Requester;
use teloxide::types::{ChatId, MessageId};

pub mod messages;

#[derive(Clone, Debug)]
pub struct NatsManager {
    nc: Client,
}

impl NatsManager {
    pub fn new(server: &str, user: &str, password: &str) -> Self {
        Self {
            nc: block_on(
                async_nats::ConnectOptions::with_user_and_password(user.into(), password.into())
                    .name("MemeBot")
                    .connect(server),
            )
            .expect("Can't connect to NATS"),
        }
    }

    pub fn subscriber(&self, bot: &Bot) {
        tokio::task::spawn({
            let client = self.nc.clone();
            let bc = bot.clone();

            async move {
                let mut subscriber = client.subscribe("statistics").await?;
                while let Some(msg) = subscriber.next().await {
                    let stats: StatisticMessage = serde_json::from_slice(&msg.payload)?;
                    let mut message = stats.message;

                    for user in stats.user_ids {
                        let info = bot::get_chat_user(&bc, stats.chat_id, user.1).await;
                        message = message.replace(&user.0, &get_user_text(&info));
                    }

                    let mut snd = bc.send_message(ChatId(stats.chat_id), message);

                    if let Some(reply_id) = stats.reply_id {
                        snd = snd.reply_to_message_id(MessageId(reply_id as i32));
                    }

                    snd.await.expect("Can't send message");
                }

                Ok::<(), async_nats::Error>(())
            }
        });
    }

    pub fn publish(&self, msg: &StatisticMessage) {
        match block_on(self.nc.publish("statistics", json!(msg).to_string().into())) {
            Ok(_) => {
                info!("Message published!")
            }
            Err(_) => {
                error!("Can't publish message to NATS!")
            }
        }
    }
}
