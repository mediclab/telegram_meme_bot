mod schema;

pub mod repository;
pub mod manager;
pub mod models;

use diesel::pg::PgConnection;
use diesel::r2d2::{ Pool, PooledConnection, ConnectionManager };

pub type PgPool = Pool<ConnectionManager<PgConnection>>;
pub type PgPooledConnection = PooledConnection<ConnectionManager<PgConnection>>;