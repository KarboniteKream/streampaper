use chrono::Utc;
use diesel::prelude::*;
use diesel::SqliteConnection;
use std::process::Command;

use super::db;
use super::schema;

pub fn update_source(source: &db::Source, conn: &SqliteConnection) {
    use schema::sources::dsl;

    let output = Command::new("youtube-dl")
        .args(["-g", "-f", "best", &source.url])
        .output()
        .expect("Failed to execute youtube-dl");

    if !output.status.success() {
        return;
    }

    if let Ok(playlist) = String::from_utf8(output.stdout) {
        let result = diesel::update(dsl::sources.find(source.id))
            .set((
                dsl::playlist.eq(playlist.trim()),
                dsl::updated_at.eq(Utc::now().timestamp()),
            ))
            .execute(conn);

        if let Err(e) = result {
            eprintln!("Unable to update source '{}': {}", source.name, e);
        }
    }
}

pub fn download_image(source: &db::Source, filename: &str) {
    if source.playlist.is_none() {
        return;
    }

    let output = Command::new("ffmpeg")
        .args([
            "-i",
            source.playlist.as_ref().unwrap(),
            "-frames:v",
            "1",
            "-qscale:v",
            "2",
            "-y",
            filename,
        ])
        .output()
        .expect("Failed to execute ffmpeg");

    if !output.status.success() {
        eprintln!("Unable to download image for '{}'", source.name);
    }
}
