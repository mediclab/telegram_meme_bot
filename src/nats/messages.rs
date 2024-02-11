use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct StatisticMessage {
    pub chat_id: i64,
    pub user_ids: Vec<(String, i64)>,
    pub message: String,
}
