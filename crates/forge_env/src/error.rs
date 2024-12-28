use derive_more::derive::{Display, From};

#[derive(Debug, Display, From)]
pub enum Error {
    Handlebars(handlebars::RenderError),
    IO(std::io::Error),
    Var(std::env::VarError),
}

pub type Result<T> = std::result::Result<T, Error>;
