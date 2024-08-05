use std::process::Command;

use crate::util::Error::{CommandError, NoPlaylist};
use crate::util::Result;

use super::db;

pub fn download(source: &db::Source, filename: &str) -> Result<()> {
    let playlist = source
        .playlist
        .as_ref()
        .ok_or_else(|| NoPlaylist(source.name.clone()))?;

    let command = "ffmpeg".to_string();
    let mut args = vec![
        "-i",
        playlist,
        "-frames:v",
        "1",
        "-qscale:v",
        "2",
        "-y",
        filename,
    ];

    if let Some(headers) = &source.headers {
        args = [vec!["-headers", headers], args].concat();
    }

    let output = Command::new(&command).args(args).output()?;

    if !output.status.success() {
        let message = String::from_utf8(output.stderr)?;
        return Err(CommandError(command, message).into());
    }

    Ok(())
}
