use diesel::pg::PgConnection;
use std::env;
use diesel::r2d2::{ Pool, PooledConnection, ConnectionManager, PoolError };

pub type PgPool = Pool<ConnectionManager<PgConnection>>;
pub type PgPooledConnection = PooledConnection<ConnectionManager<PgConnection>>;

//Connects to Postgres and call init pool
pub fn establish_connection() -> PgPool {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    init_pool(&database_url).expect("Failed to create pool")
}


//Creates a default R2D2 Postgres DB Pool
fn init_pool(database_url: &str) -> Result<PgPool, PoolError> {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder().build(manager)
}


//this functions returns a connection from the Pool
pub fn pg_pool_handler(pool: &PgPool) -> Result<PgPooledConnection, PoolError> {
    let _pool = pool.get().unwrap();
    Ok(_pool)
}