use chrono::{NaiveDateTime, Utc};
use diesel::{
    dsl,
    pg::PgConnection,
    prelude::*,
    r2d2::{ConnectionManager, Pool, PoolError, PooledConnection},
    result::Error,
    sql_types::{BigInt, Bool},
};
use uuid::Uuid;

use crate::app::utils::Period;
use crate::database::{
    models::*,
    schema::{
        chat_admins as ChatAdminsSchema, chats as ChatsSchema, meme_likes as MemeLikesSchema,
        memes as MemesSchema, users as UsersSchema,
    },
};

pub mod models;
#[rustfmt::skip]
mod schema;

pub type PgPool = Pool<ConnectionManager<PgConnection>>;
pub type PgPooledConnection = PooledConnection<ConnectionManager<PgConnection>>;

#[derive(Clone)]
pub struct DBManager {
    pool: PgPool,
}

impl DBManager {
    pub fn connect(database_url: &str) -> DBManager {
        DBManager {
            pool: Pool::builder()
                .build(ConnectionManager::<PgConnection>::new(database_url))
                .expect("Failed to create pool"),
        }
    }

    //this functions returns a connection from the Pool
    pub fn get_pool(&self) -> Result<PgPooledConnection, PoolError> {
        Ok(self.pool.get().expect("Can't get pool"))
    }

    pub fn get_connection(&self) -> PgPooledConnection {
        self.get_pool().expect("Can't get connection from pool")
    }

    pub fn add_meme(&self, meme: &AddMeme) -> Result<Meme, Error> {
        diesel::insert_into(MemesSchema::table)
            .values(meme)
            .get_result(&mut *self.get_connection())
    }

    pub fn add_meme_hashes(&self, uuid: &Uuid, long_hash: &str, short_hash: &str) -> bool {
        diesel::update(MemesSchema::table)
            .filter(MemesSchema::dsl::uuid.eq(uuid))
            .set((
                MemesSchema::dsl::long_hash.eq(long_hash),
                MemesSchema::dsl::short_hash.eq(short_hash),
                MemesSchema::dsl::updated_at.eq(Utc::now().naive_utc()),
            ))
            .execute(&mut *self.get_connection())
            .is_ok()
    }

    pub fn replace_meme_msg_id(&self, uuid: &Uuid, msg_id: i64) -> bool {
        diesel::update(MemesSchema::table)
            .filter(MemesSchema::dsl::uuid.eq(uuid))
            .set((
                MemesSchema::dsl::msg_id.eq(msg_id),
                MemesSchema::dsl::updated_at.eq(Utc::now().naive_utc()),
            ))
            .execute(&mut *self.get_connection())
            .is_ok()
    }

    pub fn get_meme(&self, uuid: &Uuid) -> Result<Meme, Error> {
        MemesSchema::table
            .find(uuid)
            .first(&mut *self.get_connection())
    }

    pub fn get_memes_by_short_hash(&self, short_hash: &str) -> Result<Vec<Meme>, Error> {
        MemesSchema::table
            .filter(MemesSchema::dsl::short_hash.eq(short_hash))
            .load(&mut *self.get_connection())
    }

    pub fn get_meme_by_msg_id(&self, msg_id: i64, chat_id: i64) -> Result<Meme, Error> {
        MemesSchema::table
            .filter(MemesSchema::dsl::msg_id.eq(msg_id))
            .filter(MemesSchema::dsl::chat_id.eq(chat_id))
            .first(&mut *self.get_connection())
    }

    pub fn delete_meme(&self, uuid: &Uuid) -> bool {
        diesel::delete(MemesSchema::table)
            .filter(MemesSchema::dsl::uuid.eq(uuid))
            .execute(&mut *self.get_connection())
            .is_ok()
    }

    pub fn add_chat_admin(&self, chat_id: i64, user_id: u64) -> bool {
        diesel::insert_into(ChatAdminsSchema::table)
            .values(&AddChatAdmin {
                chat_id,
                user_id: user_id as i64,
            })
            .execute(&mut *self.get_connection())
            .is_ok()
    }

    // pub fn is_user_chat_admin(&self, chat_id: i64, user_id: u64) -> bool {
    //     dsl::select(dsl::exists(
    //         ChatAdminsSchema::table
    //             .filter(ChatAdminsSchema::dsl::chat_id.eq(chat_id))
    //             .filter(ChatAdminsSchema::dsl::user_id.eq(user_id as i64)),
    //     ))
    //     .get_result(&mut *self.get_connection())
    //     .unwrap_or(false)
    // }

    pub fn get_admin_chats(&self, user_id: u64) -> Result<Vec<i64>, Error> {
        ChatAdminsSchema::table
            .select(ChatAdminsSchema::dsl::chat_id)
            .filter(ChatAdminsSchema::dsl::user_id.eq(user_id as i64))
            .load(&mut *self.get_connection())
    }

    pub fn like(&self, from_user_id: i64, uuid: &Uuid) -> bool {
        self.insert_for_like(from_user_id, uuid, MemeLikeOperation::Like)
    }

    pub fn dislike(&self, from_user_id: i64, uuid: &Uuid) -> bool {
        self.insert_for_like(from_user_id, uuid, MemeLikeOperation::Dislike)
    }

    pub fn cancel_like(&self, from_user_id: i64, uuid: &Uuid) -> bool {
        self.cancel_for_like(from_user_id, uuid, MemeLikeOperation::Like)
    }

    pub fn cancel_dislike(&self, from_user_id: i64, uuid: &Uuid) -> bool {
        self.cancel_for_like(from_user_id, uuid, MemeLikeOperation::Dislike)
    }

    pub fn like_exists(&self, from_user_id: i64, uuid: &Uuid) -> bool {
        self.exists_for_like(from_user_id, uuid, MemeLikeOperation::Like)
    }

    pub fn dislike_exists(&self, from_user_id: i64, uuid: &Uuid) -> bool {
        self.exists_for_like(from_user_id, uuid, MemeLikeOperation::Dislike)
    }

    pub fn count_likes(&self, uuid: &Uuid) -> i64 {
        self.count_for_like(uuid, MemeLikeOperation::Like)
    }

    pub fn count_dislikes(&self, uuid: &Uuid) -> i64 {
        self.count_for_like(uuid, MemeLikeOperation::Dislike)
    }

    pub fn get_top_meme(&self, period: &Period) -> Result<(Meme, i64), Error> {
        let (start, end) = period.dates();

        MemesSchema::table
            .left_join(MemeLikesSchema::table)
            .group_by((MemesSchema::dsl::uuid, MemesSchema::dsl::posted_at))
            .filter(MemesSchema::dsl::posted_at.ge(start.naive_utc()))
            .filter(MemesSchema::dsl::posted_at.le(end.naive_utc()))
            .select((
                MemesSchema::all_columns,
                dsl::sql::<BigInt>("SUM(\"meme_likes\".\"num\") as likes"),
            ))
            .having(dsl::sql::<Bool>("SUM(\"meme_likes\".\"num\") <> 0"))
            .order_by(dsl::sql::<BigInt>("likes DESC"))
            .then_order_by(MemesSchema::dsl::posted_at.desc())
            .first(&mut *self.get_connection())
    }

    pub fn get_top_selflikes(&self, period: &Period) -> Result<(i64, i64), Error> {
        let (start, end) = period.dates();

        MemeLikesSchema::table
            .inner_join(MemesSchema::table)
            .group_by(MemeLikesSchema::dsl::user_id)
            .filter(MemeLikesSchema::dsl::created_at.ge(start.naive_utc()))
            .filter(MemeLikesSchema::dsl::created_at.le(end.naive_utc()))
            .filter(MemeLikesSchema::dsl::user_id.eq(MemesSchema::dsl::user_id))
            .filter(MemeLikesSchema::dsl::num.eq(MemeLikeOperation::Like.id()))
            .select((
                MemeLikesSchema::dsl::user_id,
                dsl::sql::<BigInt>("SUM(\"meme_likes\".\"num\") as likes"),
            ))
            .having(dsl::sql::<Bool>("SUM(\"meme_likes\".\"num\") > 0"))
            .order_by(dsl::sql::<BigInt>("likes DESC"))
            .first(&mut *self.get_connection())
    }

    pub fn get_top_likers(
        &self,
        period: &Period,
        operation: MemeLikeOperation,
    ) -> Result<(i64, i64), Error> {
        let (start, end) = period.dates();

        MemeLikesSchema::table
            .group_by(MemeLikesSchema::dsl::user_id)
            .filter(MemeLikesSchema::dsl::created_at.ge(start.naive_utc()))
            .filter(MemeLikesSchema::dsl::created_at.le(end.naive_utc()))
            .filter(MemeLikesSchema::dsl::num.eq(operation.id()))
            .select((
                MemeLikesSchema::dsl::user_id,
                dsl::sql::<BigInt>("COUNT(\"meme_likes\".\"num\") as cnt"),
            ))
            .having(dsl::sql::<Bool>("COUNT(\"meme_likes\".\"num\") > 0"))
            .order_by(dsl::sql::<BigInt>("cnt DESC"))
            .first(&mut *self.get_connection())
    }

    pub fn get_top_memesender(&self, period: &Period) -> Result<(i64, i64), Error> {
        let (start, end) = period.dates();

        MemesSchema::table
            .group_by(MemesSchema::dsl::user_id)
            .filter(MemesSchema::dsl::posted_at.ge(start.naive_utc()))
            .filter(MemesSchema::dsl::posted_at.le(end.naive_utc()))
            .select((
                MemesSchema::dsl::user_id,
                dsl::sql::<BigInt>("COUNT(\"memes\".\"uuid\") as cnt"),
            ))
            .having(dsl::sql::<Bool>("COUNT(\"memes\".\"uuid\") > 0"))
            .order_by(dsl::sql::<BigInt>("cnt DESC"))
            .first(&mut *self.get_connection())
    }

    pub fn add_user(&self, user: &AddUser) -> Result<User, Error> {
        diesel::insert_into(UsersSchema::table)
            .values(user)
            .on_conflict(UsersSchema::dsl::user_id)
            .do_update()
            .set(UsersSchema::dsl::deleted_at.eq(None as Option<NaiveDateTime>))
            .get_result(&mut *self.get_connection())
    }

    pub fn delete_user(&self, user_id: i64) -> bool {
        diesel::update(UsersSchema::table)
            .filter(UsersSchema::dsl::user_id.eq(user_id))
            .set(UsersSchema::dsl::deleted_at.eq(Utc::now().naive_utc()))
            .execute(&mut *self.get_connection())
            .is_ok()
    }

    pub fn add_chat(&self, chat: &AddChat) -> Result<Chat, Error> {
        diesel::insert_into(ChatsSchema::table)
            .values(chat)
            .get_result(&mut *self.get_connection())
    }

    pub fn get_memes_without_hashes(&self) -> Result<Vec<Meme>, Error> {
        MemesSchema::table
            .filter(dsl::sql::<Bool>("NOT photos ? 'thumb'"))
            .filter(MemesSchema::dsl::short_hash.is_null())
            .order_by(dsl::sql::<BigInt>("posted_at DESC"))
            .limit(50)
            .load(&mut *self.get_connection())
    }

    pub fn get_users_ids_not_in_table(&self) -> Result<Vec<i64>, Error> {
        dsl::sql::<BigInt>("(SELECT DISTINCT user_id FROM memes UNION SELECT DISTINCT user_id FROM meme_likes) EXCEPT SELECT user_id FROM users")
            .load::<i64>(&mut *self.get_connection())
    }

    pub fn get_max_disliked_meme(&self, period: &Period) -> Result<(Meme, i64), Error> {
        let (start, end) = period.dates();

        MemeLikesSchema::table
            .inner_join(MemesSchema::table)
            .group_by(MemesSchema::dsl::uuid)
            .filter(MemeLikesSchema::dsl::created_at.ge(start.naive_utc()))
            .filter(MemeLikesSchema::dsl::created_at.le(end.naive_utc()))
            .filter(MemeLikesSchema::dsl::num.eq(MemeLikeOperation::Dislike.id()))
            .select((
                MemesSchema::all_columns,
                dsl::sql::<BigInt>("ABS(SUM(\"meme_likes\".\"num\")) as sum"),
            ))
            .having(dsl::sql::<Bool>("SUM(\"meme_likes\".\"num\") < -5"))
            .order_by(dsl::sql::<BigInt>("sum ASC"))
            .first(&mut *self.get_connection())
    }

    pub fn get_memes_count(&self, chat_id: i64) -> i64 {
        MemesSchema::table
            .filter(MemesSchema::dsl::chat_id.eq(chat_id))
            .count()
            .get_result(&mut *self.get_connection())
            .unwrap_or(0)
    }

    pub fn get_meme_likes_count(&self, chat_id: i64) -> i64 {
        MemeLikesSchema::table
            .inner_join(MemesSchema::table)
            .filter(MemesSchema::dsl::chat_id.eq(chat_id))
            .filter(MemeLikesSchema::dsl::num.eq(MemeLikeOperation::Like.id()))
            .count()
            .get_result(&mut *self.get_connection())
            .unwrap_or(0)
    }

    pub fn get_meme_dislikes_count(&self, chat_id: i64) -> i64 {
        MemeLikesSchema::table
            .inner_join(MemesSchema::table)
            .filter(MemesSchema::dsl::chat_id.eq(chat_id))
            .filter(MemeLikesSchema::dsl::num.eq(MemeLikeOperation::Dislike.id()))
            .count()
            .get_result(&mut *self.get_connection())
            .unwrap_or(0)
    }

    fn insert_for_like(
        &self,
        from_user_id: i64,
        uuid: &Uuid,
        operation: MemeLikeOperation,
    ) -> bool {
        diesel::insert_into(MemeLikesSchema::table)
            .values((
                MemeLikesSchema::dsl::user_id.eq(from_user_id),
                MemeLikesSchema::dsl::meme_uuid.eq(uuid),
                MemeLikesSchema::dsl::num.eq(operation.id()),
            ))
            .on_conflict((
                MemeLikesSchema::dsl::user_id,
                MemeLikesSchema::dsl::meme_uuid,
            ))
            .do_update()
            .set(MemeLikesSchema::dsl::num.eq(operation.id()))
            .execute(&mut *self.get_connection())
            .is_ok()
    }

    fn cancel_for_like(
        &self,
        from_user_id: i64,
        uuid: &Uuid,
        operation: MemeLikeOperation,
    ) -> bool {
        diesel::delete(MemeLikesSchema::table)
            .filter(MemeLikesSchema::dsl::meme_uuid.eq(uuid))
            .filter(MemeLikesSchema::dsl::user_id.eq(from_user_id))
            .filter(MemeLikesSchema::dsl::num.eq(operation.id()))
            .execute(&mut *self.get_connection())
            .is_ok()
    }

    fn exists_for_like(
        &self,
        from_user_id: i64,
        uuid: &Uuid,
        operation: MemeLikeOperation,
    ) -> bool {
        dsl::select(dsl::exists(
            MemeLikesSchema::table
                .filter(MemeLikesSchema::dsl::meme_uuid.eq(uuid))
                .filter(MemeLikesSchema::dsl::user_id.eq(from_user_id))
                .filter(MemeLikesSchema::dsl::num.eq(operation.id())),
        ))
        .get_result(&mut *self.get_connection())
        .unwrap_or(false)
    }

    fn count_for_like(&self, uuid: &Uuid, operation: MemeLikeOperation) -> i64 {
        MemeLikesSchema::table
            .filter(MemeLikesSchema::dsl::meme_uuid.eq(uuid))
            .filter(MemeLikesSchema::dsl::num.eq(operation.id()))
            .count()
            .get_result(&mut *self.get_connection())
            .unwrap_or(0)
    }
}
