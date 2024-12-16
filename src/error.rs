use derive_more::derive::From;

#[derive(Debug, From)]
pub enum Error {
    Engine(crate::core::error::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
