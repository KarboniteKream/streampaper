use rusqlite::{Connection, Row};
use std::path::Path;

pub fn open(path: &Path) -> rusqlite::Result<Connection> {
    Connection::open(path)
}

#[derive(Debug, Clone, Copy)]
pub enum SourceType {
    Url,
    YouTube,
    Stream,
    Unknown,
}

impl From<i32> for SourceType {
    fn from(value: i32) -> Self {
        match value {
            1 => Self::Url,
            2 => Self::YouTube,
            3 => Self::Stream,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Source {
    pub id: i64,
    pub name: String,
    pub typ: SourceType,
    pub url: Option<String>,
    pub playlist: Option<String>,
    pub headers: Option<String>,
}

impl Source {
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            name: row.get(1)?,
            typ: SourceType::from(row.get::<_, i32>(2)?),
            url: row.get(3)?,
            playlist: row.get(4)?,
            headers: row.get(5)?,
        })
    }
}
