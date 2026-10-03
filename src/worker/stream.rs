use std::path::Path;
use std::process::Command;

use crate::util::Error::{CommandError, NoPlaylist};
use crate::util::Result;

use super::db;

pub fn download(source: &db::Source, filename: &Path) -> Result<()> {
    let playlist = source
        .playlist
        .as_ref()
        .ok_or_else(|| NoPlaylist(source.name.clone()))?;

    let command = "ffmpeg";
    let mut cmd = Command::new(command);

    if let Some(headers) = &source.headers {
        cmd.args(["-headers", headers]);
    }

    cmd.args(["-i", playlist, "-frames:v", "1", "-qscale:v", "2", "-y"]);
    cmd.arg(filename);

    let output = cmd.output()?;

    if !output.status.success() {
        let message = String::from_utf8(output.stderr)?;
        return Err(CommandError(command.to_string(), message).into());
    }

    Ok(())
}
