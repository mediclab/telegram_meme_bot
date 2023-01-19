use core::fmt::Debug;
use serde::{Deserialize, Serialize};
use serde_json::json;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub enum CallbackOperations {
    Like,
    Dislike,
    Delete,
    None,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MemeCallback {
    pub uuid: Uuid,
    pub op: CallbackOperations,
}

pub struct MemeMarkup {
    likes: i64,
    dislikes: i64,
    uuid: Uuid,
}

impl MemeMarkup {
    pub fn new(likes: i64, dislikes: i64, uuid: Uuid) -> Self {
        Self {
            likes,
            dislikes,
            uuid,
        }
    }

    pub fn get_markup(&self) -> InlineKeyboardMarkup {
        InlineKeyboardMarkup::new(vec![vec![
            InlineKeyboardButton::callback(
                format!(
                    "{} Like ({})",
                    emojis::get_by_shortcode("heart").unwrap().as_str(),
                    self.likes
                ),
                json!(MemeCallback {
                    uuid: self.uuid,
                    op: CallbackOperations::Like
                })
                .to_string(),
            ),
            InlineKeyboardButton::callback(
                format!(
                    "{} Dislike ({})",
                    emojis::get_by_shortcode("broken_heart").unwrap().as_str(),
                    self.dislikes
                ),
                json!(MemeCallback {
                    uuid: self.uuid,
                    op: CallbackOperations::Dislike
                })
                .to_string(),
            ),
        ]])
    }
}

pub struct AccordionMarkup {
    uuid: Uuid,
}

impl AccordionMarkup {
    pub fn new(uuid: Uuid) -> Self {
        Self { uuid }
    }

    pub fn get_markup(&self) -> InlineKeyboardMarkup {
        InlineKeyboardMarkup::new(vec![
            vec![InlineKeyboardButton::callback(
                format!(
                    "{} Беру на себя ответственность",
                    emojis::get_by_shortcode("thumbsup").unwrap().as_str()
                ),
                json!(MemeCallback {
                    uuid: self.uuid,
                    op: CallbackOperations::None
                })
                .to_string(),
            )],
            vec![InlineKeyboardButton::callback(
                format!(
                    "{} Удалите, прошу прощения",
                    emojis::get_by_shortcode("thumbsdown").unwrap().as_str()
                ),
                json!(MemeCallback {
                    uuid: self.uuid,
                    op: CallbackOperations::Delete
                })
                .to_string(),
            )],
        ])
    }
}

pub struct DeleteMarkup {
    uuid: Uuid,
}

impl DeleteMarkup {
    pub fn new(uuid: Uuid) -> Self {
        Self { uuid }
    }

    pub fn get_markup(&self) -> InlineKeyboardMarkup {
        InlineKeyboardMarkup::new(vec![
            vec![InlineKeyboardButton::callback(
                format!(
                    "{} Нет, я передумал(а)",
                    emojis::get_by_shortcode("x").unwrap().as_str()
                ),
                json!(MemeCallback {
                    uuid: self.uuid,
                    op: CallbackOperations::None
                })
                .to_string(),
            )],
            vec![InlineKeyboardButton::callback(
                format!(
                    "{} Да, я хочу удалить",
                    emojis::get_by_shortcode("wastebasket").unwrap().as_str()
                ),
                json!(MemeCallback {
                    uuid: self.uuid,
                    op: CallbackOperations::Delete
                })
                .to_string(),
            )],
        ])
    }
}
