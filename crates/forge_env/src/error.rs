use derive_more::derive::From;

#[derive(From)]
pub enum Error {
    IO(std::io::Error),
    Handlebars(handlebars::RenderError),
}

pub type Result<T> = std::result::Result<T, Error>;
