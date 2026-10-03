use std::fs;
use std::path::Path;

use crate::util::Error::NoUrl;
use crate::util::Result;

use super::db;

pub fn download(source: &db::Source, filename: &Path) -> Result<()> {
    let url = source
        .url
        .as_ref()
        .ok_or_else(|| NoUrl(source.name.clone()))?;

    let bytes = ureq::get(url).call()?.into_body().read_to_vec()?;

    fs::write(filename, bytes)?;
    Ok(())
}
