use serde_json::json;
use teloxide::types::User;
use uuid::Uuid;

use crate::database::manager::DBManager;
use crate::database::PgPooledConnection;

use crate::database::models::*;
use crate::database::schema::memes as MemesSchema;
use crate::database::schema::meme_likes as MemeLikesSchema;

use diesel::{result::Error, dsl, prelude::*};
use teloxide::types::Message;
//use diesel::query_dsl::RunQueryDsl;

pub struct MemeRepository {
    db_manager: DBManager,
}

trait Repository {
    fn get_connection(&self) -> PgPooledConnection;
}

impl Repository for MemeRepository {
    fn get_connection(&self) -> PgPooledConnection {
        self.db_manager.get_pool().expect("Cannot get connection from pool")
    }
}

impl MemeRepository {
    pub fn new(db_manager: DBManager) -> Self {
        Self { db_manager }
    }

    pub fn add(&self, message: &Message) -> Result<Meme, Error> {
        let user_id = message.from().unwrap().id.0 as i64;

        diesel::insert_into(MemesSchema::table)
            .values(
                (
                    MemesSchema::dsl::user_id.eq(user_id),
                    MemesSchema::dsl::chat_id.eq(message.chat.id.0),
                    MemesSchema::dsl::photos.eq(Some(json!(message.photo()))),
                )
            )
            .get_result::<Meme>(&mut *self.get_connection())
    }

    pub fn add_msg_id(&self, uuid: &Uuid, msg: &Message) -> bool {
        let msg_id = msg.id.0 as i64;

        diesel::update(MemesSchema::table)
            .filter(MemesSchema::dsl::uuid.eq(uuid))
            .set(MemesSchema::dsl::msg_id.eq(msg_id))
            .execute(&mut *self.get_connection())
            .is_ok()
    }

    pub fn get(&self, uuid: &Uuid) -> Result<Meme, Error> {
        MemesSchema::table.find(uuid).first(&mut *self.get_connection())
    }

    pub fn get_by_msg_id(&self, msg_id: i64, chat_id: i64) -> Result<Meme, Error> {
        MemesSchema::table
            .filter(MemesSchema::dsl::msg_id.eq(msg_id))
            .filter(MemesSchema::dsl::chat_id.eq(chat_id))
            .first(&mut *self.get_connection())
    }
}

pub struct MemeLikeRepository {
    db_manager: DBManager,
}

impl Repository for MemeLikeRepository {
    fn get_connection(&self) -> PgPooledConnection {
        self.db_manager.get_pool().expect("Cannot get connection from pool")
    }
}

impl MemeLikeRepository {
    pub fn new(db_manager: DBManager) -> Self {
        Self { db_manager }
    }

    pub fn like(&self, from_user: &User, uuid: &Uuid) -> bool {
        let user_id = from_user.id.0 as i64;

        if self.exists(from_user, uuid) {
            diesel::update(MemeLikesSchema::table)
                .filter(MemeLikesSchema::dsl::user_id.eq(user_id))
                .filter(MemeLikesSchema::dsl::meme_uuid.eq(uuid))
                .set(MemeLikesSchema::dsl::num.eq(1))
                .execute(&mut *self.get_connection())
                .is_ok()
        } else {
            diesel::insert_into(MemeLikesSchema::table)
            .values(
                (
                    MemeLikesSchema::dsl::user_id.eq(user_id),
                    MemeLikesSchema::dsl::meme_uuid.eq(uuid),
                    MemeLikesSchema::dsl::num.eq(1)
                )
            )
            .execute(&mut *self.get_connection())
            .is_ok()
        }
    }

    pub fn dislike(&self, from_user: &User, uuid: &Uuid) -> bool {
        let user_id = from_user.id.0 as i64;

        if self.exists(from_user, uuid) {
            diesel::update(MemeLikesSchema::table)
                .filter(MemeLikesSchema::dsl::user_id.eq(user_id))
                .filter(MemeLikesSchema::dsl::meme_uuid.eq(uuid))
                .set(MemeLikesSchema::dsl::num.eq(-1))
                .execute(&mut *self.get_connection())
                .is_ok()
        } else {
            diesel::insert_into(MemeLikesSchema::table)
            .values(
                (
                    MemeLikesSchema::dsl::user_id.eq(user_id),
                    MemeLikesSchema::dsl::meme_uuid.eq(uuid),
                    MemeLikesSchema::dsl::num.eq(-1)
                )
            )
            .execute(&mut *self.get_connection())
            .is_ok()
        }
    }

    pub fn exists(&self, from_user: &User, uuid: &Uuid) -> bool {
        let user_id = from_user.id.0 as i64;

        dsl::select(dsl::exists(
            MemeLikesSchema::table
            .filter(MemeLikesSchema::dsl::meme_uuid.eq(uuid))
            .filter(MemeLikesSchema::dsl::user_id.eq(user_id))
        )).get_result(&mut *self.get_connection())
        .unwrap_or(false)
    }

    pub fn count_likes(&self, uuid: &Uuid) -> i64 {
        MemeLikesSchema::table
            .filter(MemeLikesSchema::dsl::meme_uuid.eq(uuid))
            .filter(MemeLikesSchema::dsl::num.eq(1))
            .count()
            .get_result(&mut *self.get_connection())
            .unwrap_or(0)
    }

    pub fn count_dislikes(&self, uuid: &Uuid) -> i64 {
        MemeLikesSchema::table
            .filter(MemeLikesSchema::dsl::meme_uuid.eq(uuid))
            .filter(MemeLikesSchema::dsl::num.eq(-1))
            .count()
            .get_result(&mut *self.get_connection())
            .unwrap_or(0)
    }
}