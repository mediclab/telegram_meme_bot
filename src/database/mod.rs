use anyhow::Result;
use migration::{Migrator, MigratorTrait};
use once_cell::sync::OnceCell;
use sea_orm::{ConnectOptions, Database as SeaDatabase, DatabaseConnection};

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
            .sqlx_logging_level(log::LevelFilter::Debug)
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
