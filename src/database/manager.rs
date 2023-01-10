use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PoolError};

use crate::database::PgPool;
use crate::database::PgPooledConnection;

#[derive(Clone)]
pub struct DBManager {
    pool: PgPool,
}

impl DBManager {
    pub fn connect(database_url: String) -> DBManager {
        DBManager {
            pool: Pool::builder()
                .build(ConnectionManager::<PgConnection>::new(database_url))
                .expect("Failed to create pool"),
        }
    }

    //this functions returns a connection from the Pool
    pub fn get_pool(&self) -> Result<PgPooledConnection, PoolError> {
        Ok(self.pool.get().expect("Can't get connection from pool"))
    }
}
