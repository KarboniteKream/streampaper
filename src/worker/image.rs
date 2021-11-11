use std::fs;

use crate::util::Result;

use super::db;

pub fn download(source: &db::Source, filename: &str) -> Result<()> {
    let response = reqwest::blocking::get(&source.url)?;
    let bytes = response.bytes()?;

    fs::write(filename, bytes)?;
    Ok(())
}
