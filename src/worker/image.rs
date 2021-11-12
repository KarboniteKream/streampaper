use std::fs;

use crate::util::Error::NoUrl;
use crate::util::Result;

use super::db;

pub fn download(source: &db::Source, filename: &str) -> Result<()> {
    let url = source
        .url
        .as_ref()
        .ok_or_else(|| NoUrl(source.name.clone()))?;

    let response = reqwest::blocking::get(url)?;
    let bytes = response.bytes()?;

    fs::write(filename, bytes)?;
    Ok(())
}
