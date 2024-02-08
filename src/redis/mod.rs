use redis::{Client as RedisClient, Commands, Connection, RedisResult};
use serde_json::json;

#[derive(Clone)]
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

    pub fn can_send_message(&self, key: &str, chat_id: i64, message_id: i32) -> bool {
        let r_message_id = self.get_connection().get(&format!("{chat_id}_msg_{key}")).unwrap_or(0);

        if r_message_id == 0 || (message_id - r_message_id > 20) {
            let _: () = self
                .get_connection()
                .set_ex(&format!("{chat_id}_msg_{key}"), message_id, 15 * 60)
                .unwrap_or_default();

            return true;
        }

        false
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

    pub fn get_app_version(&self) -> Option<String> {
        self.get_connection().get("app_version").unwrap_or(None)
    }

    pub fn set_app_version(&self, version: &str) {
        self.get_connection()
            .set("app_version", version)
            .expect("Can't set app version")
    }

    fn get_connection(&self) -> Connection {
        self.client.get_connection().expect("Can't get connection")
    }
}
