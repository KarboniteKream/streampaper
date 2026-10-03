use std::error;
use std::fmt::{self, Display, Formatter};

use std::time::{SystemTime, UNIX_EPOCH};

use crate::models::SourceType;

pub type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

pub fn unix_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
pub enum Error {
    CommandError(String, String),
    NoPlaylist(String),
    NoUrl(String),
    UnsupportedSource(SourceType),
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::CommandError(command, message) => {
                write!(f, "Unable to execute command '{}': {}", command, message)
            }
            Self::NoPlaylist(source) => write!(f, "Source '{}' has no playlist", source),
            Self::NoUrl(source) => write!(f, "Source '{}' has no URL", source),
            Self::UnsupportedSource(typ) => write!(f, "Source type '{:?}' is not supported", typ),
        }
    }
}
