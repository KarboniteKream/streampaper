use std::error;
use std::fmt::{self, Display, Formatter};

use crate::models::SourceType;

pub type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
pub enum Error {
    CommandError(String, String),
    DatabaseError(diesel::result::Error),
    NoPlaylist(String),
    UnsupportedSource(SourceType),
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::CommandError(command, message) => {
                write!(f, "Unable to execute command '{}': {}", command, message)
            }
            Self::DatabaseError(error) => write!(f, "Database error: {}", error),
            Self::NoPlaylist(source) => write!(f, "Source '{}' has no playlist", source),
            Self::UnsupportedSource(typ) => write!(f, "Source type '{:?}' is not supported", typ),
        }
    }
}

impl From<diesel::result::Error> for Error {
    fn from(e: diesel::result::Error) -> Error {
        Self::DatabaseError(e)
    }
}
