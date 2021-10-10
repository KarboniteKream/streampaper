use std::fs;

use super::db;

pub fn download(source: &db::Source, filename: &str) {
    let response = reqwest::blocking::get(&source.url);

    if let Err(e) = response {
        eprintln!("Unable to download image for '{}': {}", source.name, e);
        return;
    }

    if let Ok(bytes) = response.unwrap().bytes() {
        fs::write(filename, bytes)
            .unwrap_or_else(|e| eprintln!("Unable to write '{}': {}", filename, e));
    }
}
