use std::path::Path;
use std::process::Command;

use crate::util::Error::{CommandFailed, NoUrl};
use crate::util::Result;

use super::db;

pub fn download(source: &db::Source, filename: &Path) -> Result<()> {
    let url = source
        .url
        .as_ref()
        .ok_or_else(|| NoUrl(source.name.clone()))?;

    let command = "curl";
    let mut cmd = Command::new(command);

    if let Some(headers) = &source.headers {
        for header in headers.split(',') {
            cmd.args(["-H", header]);
        }
    }

    cmd.args(["-s", "-f", "-L", "-o"]);
    cmd.arg(filename);
    cmd.arg(url);

    let output = cmd.output()?;

    if !output.status.success() {
        let message = String::from_utf8_lossy(&output.stderr).into_owned();
        return Err(CommandFailed(command.to_string(), message).into());
    }

    Ok(())
}
