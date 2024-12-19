use derive_more::derive::From;

#[derive(Debug, From)]
pub enum Error {
    Serde(serde_json::Error),
}

pub(crate) type Result<T> = std::result::Result<T, Error>;
