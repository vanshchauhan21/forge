use std::fmt::{Debug, Display, Formatter};
use std::io;

#[derive(Debug)]
pub enum Error {
    DirectoryAccess(String),
    DirectoryRead(io::Error),
    FileRead { path: String, error: io::Error },
    UnsupportedLanguage(String),
    Query(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::DirectoryAccess(msg) => write!(f, "{}", msg),
            Error::DirectoryRead(e) => write!(f, "Failed to read directory: {}", e),
            Error::FileRead { path, error } => write!(f, "Failed to read file {}: {}", path, error),
            Error::UnsupportedLanguage(msg) => write!(f, "Language error: {}", msg),
            Error::Query(msg) => write!(f, "Query error: {}", msg),
        }
    }
}
