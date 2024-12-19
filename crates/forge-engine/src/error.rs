#[derive(Debug, thiserror::Error)]
pub enum Error {}

pub(crate) type Result<T> = std::result::Result<T, Error>;
