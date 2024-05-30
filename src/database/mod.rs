use anyhow::Result;
use diesel::{
    pg::PgConnection,
    prelude::*,
    r2d2::{ConnectionManager, Pool, PoolError, PooledConnection},
    result::Error,
};
use migration::{Migrator, MigratorTrait};
use once_cell::sync::OnceCell;
use sea_orm::{ConnectOptions, Database as SeaDatabase, DatabaseConnection};
use uuid::Uuid;

use crate::database::{
    models::*,
    schema::{chat_admins as ChatAdminsSchema, memes as MemesSchema},
};

pub mod models;
#[rustfmt::skip]
mod schema;

pub mod entity;

pub static INSTANCE: OnceCell<Database> = OnceCell::new();

#[derive(Clone, Debug)]
pub struct Database {
    connection: DatabaseConnection,
}

impl Database {
    pub async fn new(database_url: &str) -> Self {
        let mut opts = ConnectOptions::new(database_url);

        opts.max_connections(100)
            .min_connections(5)
            .sqlx_logging(true)
            .sqlx_logging_level(log::LevelFilter::Info)
            .set_schema_search_path("public");

        let connection = SeaDatabase::connect(opts).await.expect("Can't connect to database");

        Self { connection }
    }

    pub fn connection(&self) -> &DatabaseConnection {
        &self.connection
    }

    pub fn global() -> &'static Database {
        INSTANCE.get().expect("Database is not initialized")
    }

    pub async fn migrate(&self) -> Result<()> {
        Migrator::up(&self.connection, None).await?;

        Ok(())
    }
}

pub type PgPool = Pool<ConnectionManager<PgConnection>>;
pub type PgPooledConnection = PooledConnection<ConnectionManager<PgConnection>>;

#[derive(Clone, Debug)]
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

    pub fn get_memes_by_short_hash(&self, short_hash: &str) -> Result<Vec<Meme>, Error> {
        MemesSchema::table
            .filter(MemesSchema::dsl::short_hash.eq(short_hash))
            .load(&mut *self.get_connection())
    }

    pub fn get_admin_chats(&self, user_id: u64) -> Result<Vec<i64>, Error> {
        ChatAdminsSchema::table
            .select(ChatAdminsSchema::dsl::chat_id)
            .filter(ChatAdminsSchema::dsl::user_id.eq(user_id as i64))
            .load(&mut *self.get_connection())
    }
}
