use std::fmt::{Debug, Display, Formatter};
use std::io;

#[derive(Debug)]
pub enum Error {
    DirectoryAccess(String),
    DirectoryRead(io::Error),
    FileRead { path: String, error: io::Error },
    LanguageError(String),
    QueryError(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::DirectoryAccess(msg) => write!(f, "{}", msg),
            Error::DirectoryRead(e) => write!(f, "Failed to read directory: {}", e),
            Error::FileRead { path, error } => {
                write!(f, "Failed to read file {}: {}", path, error)
            }
            Error::LanguageError(msg) => write!(f, "Language error: {}", msg),
            Error::QueryError(msg) => write!(f, "Query error: {}", msg),
        }
    }
}
