use std::path::StripPrefixError;

use derive_more::derive::{Display, From};

#[derive(Display, From)]
pub enum Error {
    Inquire(#[from] inquire::InquireError),
    Tokio(#[from] tokio::io::Error),
    JoinError(#[from] tokio::task::JoinError),
    Parse(String),
    Ignore(#[from] ignore::Error),
    StripPrefix(#[from] StripPrefixError),
}

pub type Result<A> = std::result::Result<A, Error>;
