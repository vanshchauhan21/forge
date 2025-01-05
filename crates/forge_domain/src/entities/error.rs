use derive_more::derive::From;

#[derive(From, Debug)]
pub enum Error {
    ToolUseMissingName,
    Serde(serde_json::Error),
}

pub type Result<A> = std::result::Result<A, Error>;
