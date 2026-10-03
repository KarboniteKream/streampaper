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

    let mut reader = ureq::get(url).call()?.into_body().into_reader();
    let mut file = fs::File::create(filename)?;
    std::io::copy(&mut reader, &mut file)?;

    Ok(())
}
