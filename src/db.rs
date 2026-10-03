use rusqlite::{Connection, Row};
use std::path::Path;

pub fn open(path: &Path) -> rusqlite::Result<Connection> {
    Connection::open(path)
}

#[derive(Debug, Clone)]
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

impl Source {
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            name: row.get(1)?,
            typ: row.get(2)?,
            url: row.get(3)?,
            playlist: row.get(4)?,
            headers: row.get(5)?,
            enabled: row.get(6)?,
            updated_at: row.get(7)?,
        })
    }
}
