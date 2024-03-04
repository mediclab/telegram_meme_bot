use serde::{Deserialize, Serialize};
use teloxide::types::{ChatId, MessageId};

#[derive(Serialize, Deserialize, Debug)]
pub struct StatisticMessage {
    pub chat_id: i64,
    pub user_ids: Vec<(String, i64)>,
    pub reply_id: Option<i64>,
    pub message: String,
}

impl StatisticMessage {
    pub fn reply_message(&self) -> Option<MessageId> {
        if let Some(id) = self.reply_id {
            return Some(MessageId(id as i32));
        }

        None
    }

    pub fn chat(&self) -> ChatId {
        ChatId(self.chat_id)
    }
}
