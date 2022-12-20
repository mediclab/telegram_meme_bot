use diesel::pg::PgConnection;
use diesel::r2d2::{ Pool, PooledConnection, ConnectionManager, PoolError };

pub type PgPool = Pool<ConnectionManager<PgConnection>>;
pub type PgPooledConnection = PooledConnection<ConnectionManager<PgConnection>>;

pub struct DBManager {
    pool: PgPool
}

impl DBManager {
    pub fn connect(database_url: String) -> DBManager {
        DBManager {
            pool: Pool::builder().build(
                ConnectionManager::<PgConnection>::new(database_url)
            ).expect("Failed to create pool")
        }
    }

    //this functions returns a connection from the Pool
    pub fn get_pool(&self) -> Result<PgPooledConnection, PoolError> {
        Ok(self.pool.get().unwrap())
    }
}