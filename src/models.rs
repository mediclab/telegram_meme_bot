use crate::schema::*;
use chrono::prelude::*;
use diesel::{prelude::*, sql_types::{Timestamp, Jsonb}};

#[derive(Debug, Queryable, Identifiable)]
pub struct Meme {
    pub id: i64,
    pub user_id: i64,
    pub posted_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[table_name="memes"]
pub struct NewMeme<'x> {
    pub user_id: i64,
    pub photos: &'x Jsonb
}

#[derive(Queryable, Insertable, Associations, Identifiable)]
//#[table_name="meme_likes"]
#[belongs_to(Meme)]
pub struct MemeLikes {
    pub id: i64,
    pub user_id: i64,
    pub meme_id: i64,
    pub num: u8,
    pub created_at: String,
}