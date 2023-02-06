use redis::{Client as RedisClient, Commands, Connection, RedisResult};
use serde_json::json;

pub struct RedisManager {
    client: RedisClient,
}

impl RedisManager {
    pub fn connect(redis_url: &str) -> Self {
        Self {
            client: RedisClient::open(redis_url).expect("Redis is not connected"),
        }
    }

    pub fn is_chat_registered(&self, chat_id: i64) -> bool {
        self.get_connection()
            .exists(&format!("{chat_id}_registered"))
            .unwrap_or(false)
    }

    pub fn register_chat(&self, chat_id: i64) {
        self.get_connection()
            .set(&format!("{chat_id}_registered"), true)
            .expect("Can't register chat")
    }

    pub fn set_chat_admins(&self, chat_id: i64, admins_uids: &Vec<u64>) -> bool {
        self.get_connection()
            .set(&format!("{chat_id}_admins"), json!(admins_uids).to_string())
            .expect("Can't set chat admins")
    }

    pub fn get_chat_admins(&self, chat_id: i64) -> Vec<u64> {
        let json: RedisResult<String> = self.get_connection().get(&format!("{chat_id}_admins"));

        if json.is_err() {
            return Vec::default();
        }

        serde_json::from_str::<Vec<u64>>(&json.unwrap()).unwrap_or_default()
    }

    fn get_connection(&self) -> Connection {
        self.client.get_connection().expect("Can't get connection")
    }
}
