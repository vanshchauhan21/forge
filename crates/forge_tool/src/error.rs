pub enum Error {
    ToolNotFound(String),
}

pub type Result<T> = std::result::Result<T, Error>;
