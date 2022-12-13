use crate::schema::*;
use chrono::prelude::*;
use diesel::prelude::*;

#[derive(Debug, Queryable, Identifiable)]
pub struct Meme {
    pub id: i64,
    pub user_id: i64,
    pub posted_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Queryable, Insertable)]
#[diesel(table_name = memes)]
pub struct AddMeme {
    pub user_id: i64,
    pub chat_id: i64,
    pub photos: Option<serde_json::Value>,
}

#[derive(Debug, Queryable, Insertable, Associations)]
#[diesel(table_name = meme_likes)]
#[diesel(belongs_to(Meme))]
pub struct AddLike {
    pub user_id: i64,
    pub meme_id: i64,
    pub num: i16,
}