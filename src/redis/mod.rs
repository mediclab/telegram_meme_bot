use redis::{Client as RedisClient, Commands, Connection};

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

    fn get_connection(&self) -> Connection {
        self.client.get_connection().expect("Can't get connection")
    }
}
