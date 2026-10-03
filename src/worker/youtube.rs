use diesel::SqliteConnection;
use diesel::prelude::*;
use std::process::Command;

use crate::util;
use crate::util::Error::{CommandError, NoUrl};
use crate::util::Result;

use super::db;
use super::schema;

pub fn update(source: &db::Source, conn: &mut SqliteConnection) -> Result<()> {
    use schema::sources::dsl;

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
    diesel::update(dsl::sources.find(source.id))
        .set((
            dsl::playlist.eq(playlist.trim()),
            dsl::updated_at.eq(util::unix_timestamp()),
        ))
        .execute(conn)?;

    Ok(())
}
