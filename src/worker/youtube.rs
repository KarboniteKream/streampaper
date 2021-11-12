use chrono::Utc;
use diesel::prelude::*;
use diesel::SqliteConnection;
use std::process::Command;

use crate::util::Error::{CommandError, NoUrl};
use crate::util::Result;

use super::db;
use super::schema;

pub fn update(source: &db::Source, conn: &SqliteConnection) -> Result<()> {
    use schema::sources::dsl;

    let url = source
        .url
        .as_ref()
        .ok_or_else(|| NoUrl(source.name.clone()))?;

    let command = "youtube-dl".to_string();
    let output = Command::new(&command)
        .args(["-g", "-f", "best", url])
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
