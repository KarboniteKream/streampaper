use rusqlite::Connection;
use std::process::Command;

use crate::util;
use crate::util::Error::{CommandError, NoUrl};
use crate::util::Result;

use super::db;

pub fn update(source: &db::Source, conn: &Connection) -> Result<()> {
    let url = source
        .url
        .as_ref()
        .ok_or_else(|| NoUrl(source.name.clone()))?;

    let command = "yt-dlp";
    let mut cmd = Command::new(command);

    if let Some(headers) = &source.headers {
        for header in headers.split(',') {
            cmd.args(["--add-headers", header]);
        }
    }

    cmd.args(["--get-url", "--format", "bestvideo", url]);

    let output = cmd.output()?;

    if !output.status.success() {
        let message = String::from_utf8(output.stderr)?;
        return Err(CommandError(command.to_string(), message).into());
    }

    let playlist = String::from_utf8(output.stdout)?;
    conn.execute(
        "UPDATE sources SET playlist = ?1, updated_at = ?2 WHERE id = ?3",
        (playlist.trim(), util::unix_timestamp(), source.id),
    )?;

    Ok(())
}
