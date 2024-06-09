use super::types::*;
use serde_json::json;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use uuid::Uuid;

pub struct MemeMarkup {
    likes: i64,
    dislikes: i64,
    uuid: Uuid,
}

impl MemeMarkup {
    pub fn new(likes: i64, dislikes: i64, uuid: Uuid) -> Self {
        Self { likes, dislikes, uuid }
    }

    pub fn get_markup(&self) -> InlineKeyboardMarkup {
        InlineKeyboardMarkup::new(vec![vec![
            InlineKeyboardButton::callback(
                format!("❤️ Like ({})", self.likes),
                json!(MemeCallback {
                    uuid: self.uuid,
                    op: CallbackOperations::Like
                })
                .to_string(),
            ),
            InlineKeyboardButton::callback(
                format!("💔 Dislike ({})", self.dislikes),
                json!(MemeCallback {
                    uuid: self.uuid,
                    op: CallbackOperations::Dislike
                })
                .to_string(),
            ),
        ]])
    }
}

pub struct DeleteMarkup {
    uuid: Uuid,
    ok_text: Option<String>,
    none_text: Option<String>,
}

impl DeleteMarkup {
    pub fn new(uuid: Uuid) -> Self {
        Self {
            uuid,
            ok_text: None,
            none_text: None,
        }
    }

    pub fn set_ok_text(mut self, text: &str) -> Self {
        self.ok_text = Some(text.to_string());
        self
    }

    pub fn set_none_text(mut self, text: &str) -> Self {
        self.none_text = Some(text.to_string());
        self
    }

    pub fn get_markup(&self) -> InlineKeyboardMarkup {
        InlineKeyboardMarkup::new(vec![
            vec![InlineKeyboardButton::callback(
                self.none_text.to_owned().unwrap_or(String::from("None")),
                json!(MemeCallback {
                    uuid: self.uuid,
                    op: CallbackOperations::None
                })
                .to_string(),
            )],
            vec![InlineKeyboardButton::callback(
                self.ok_text.to_owned().unwrap_or(String::from("Delete")),
                json!(MemeCallback {
                    uuid: self.uuid,
                    op: CallbackOperations::Delete
                })
                .to_string(),
            )],
        ])
    }
}
