use chrono::Utc;
use diesel::prelude::*;
use diesel::SqliteConnection;
use std::process::Command;

use crate::util::Error::{CommandError, NoPlaylist};
use crate::util::Result;

use super::db;
use super::schema;

pub fn update(source: &db::Source, conn: &SqliteConnection) -> Result<()> {
    use schema::sources::dsl;

    let command = "youtube-dl".to_string();
    let output = Command::new(&command)
        .args(["-g", "-f", "best", &source.url])
        .output()?;

    if !output.status.success() {
        let message = String::from_utf8(output.stderr)?;
        return Err(CommandError(command, message).into());
    }

    let playlist = String::from_utf8(output.stdout)?;
    diesel::update(dsl::sources.find(source.id))
        .set((
            dsl::playlist.eq(playlist.trim()),
            dsl::updated_at.eq(Utc::now().timestamp()),
        ))
        .execute(conn)?;

    Ok(())
}

pub fn download(source: &db::Source, filename: &str) -> Result<()> {
    let playlist = source
        .playlist
        .as_ref()
        .ok_or_else(|| NoPlaylist(source.name.clone()))?;

    let command = "ffmpeg".to_string();
    let output = Command::new(&command)
        .args([
            "-i",
            playlist,
            "-frames:v",
            "1",
            "-qscale:v",
            "2",
            "-y",
            filename,
        ])
        .output()?;

    if !output.status.success() {
        let message = String::from_utf8(output.stderr)?;
        return Err(CommandError(command, message).into());
    }

    Ok(())
}
