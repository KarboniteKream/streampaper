use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::SqliteConnection;

#[derive(Clone)]
pub struct ConnectionPool(Pool<ConnectionManager<SqliteConnection>>);

impl ConnectionPool {
    pub fn new(database_url: &str) -> ConnectionPool {
        let manager = ConnectionManager::new(database_url);
        ConnectionPool(Pool::new(manager).expect("Unable to create a connection pool"))
    }

    pub fn get(&self) -> PooledConnection<ConnectionManager<SqliteConnection>> {
        self.0.get().unwrap()
    }
}

#[derive(Queryable)]
pub struct Source {
    pub id: i64,
    pub name: String,
    pub typ: i32,
    pub url: Option<String>,
    pub playlist: Option<String>,
    pub headers: Option<String>,
    pub enabled: bool,
    #[allow(unused)]
    pub updated_at: i64,
}

#[derive(Queryable)]
pub struct Image {
    #[allow(unused)]
    pub id: i64,
    pub source_id: i64,
    pub timestamp: i64,
}
