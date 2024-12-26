use derive_more::derive::From;

#[derive(Debug, From)]
pub enum Error {
    Handlebars(handlebars::RenderError),
}

pub type Result<T> = std::result::Result<T, Error>;
