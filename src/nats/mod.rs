use crate::app::utils::get_user_text;
use crate::bot::BotManager;
use crate::nats::messages::StatisticMessage;
use async_nats::Client;
use envconfig::Envconfig;
use futures::executor::block_on;
use futures::StreamExt;
use serde_json::json;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Requester;

pub mod messages;

#[derive(Envconfig, Clone, Debug)]
pub struct NatsConfig {
    #[envconfig(from = "NATS_SERVER")]
    pub server: String,
    #[envconfig(from = "NATS_USER")]
    pub user: String,
    #[envconfig(from = "NATS_PASSWORD")]
    pub password: String,
}

#[derive(Clone, Debug)]
pub struct NatsManager {
    nc: Client,
}

const TOPIC_NAME: &str = "statistics";

impl NatsManager {
    pub fn new(config: &NatsConfig) -> Self {
        Self {
            nc: block_on(
                async_nats::ConnectOptions::with_user_and_password(config.user.clone(), config.password.clone())
                    .name("MemeBot")
                    .connect(&config.server),
            )
            .expect("Can't connect to NATS"),
        }
    }

    pub fn subscriber(&self, bot_manager: &BotManager) {
        tokio::task::spawn({
            let client = self.nc.clone();
            let bot = bot_manager.clone();

            async move {
                let mut subscriber = client.subscribe(TOPIC_NAME).await?;
                while let Some(msg) = subscriber.next().await {
                    let stats: StatisticMessage = serde_json::from_slice(&msg.payload)?;
                    let mut message = stats.message.clone();

                    for user in &stats.user_ids {
                        let info = bot.get_chat_user(stats.chat_id, user.1).await;
                        message = message.replace(&user.0, &get_user_text(&info));
                    }

                    let mut snd = bot.get().send_message(stats.chat(), message);

                    if let Some(reply) = stats.reply_message() {
                        snd = snd.reply_to_message_id(reply);
                    }

                    if snd.await.is_err() {
                        error!("Can't send statistic message from NATS")
                    }
                }

                Ok::<(), async_nats::Error>(())
            }
        });
    }

    pub async fn publish(&self, msg: &StatisticMessage) {
        if self
            .nc
            .publish(TOPIC_NAME, json!(msg).to_string().into())
            .await
            .is_err()
        {
            error!("Can't publish message to NATS!")
        }
    }
}
