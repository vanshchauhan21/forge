use std::fmt::{Debug, Display, Formatter};

use derive_more::derive::From;

#[derive(From)]
pub enum Error {
    Engine(crate::core::error::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&Errata::from(self), f)
    }
}

#[derive(Clone, derive_setters::Setters)]
pub struct Errata {
    pub title: String,
    #[setters(strip_option, into)]
    pub description: Option<String>,
}

impl Errata {
    fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: None,
        }
    }
}

impl Display for Errata {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.title)
    }
}

impl Debug for Errata {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ERROR {}", self.title)?;
        if let Some(desc) = &self.description {
            if !desc.trim().is_empty() {
                write!(f, "\n{}", desc)?;
            }
        }
        Ok(())
    }
}

impl From<&Error> for Errata {
    fn from(error: &Error) -> Self {
        match error {
            Error::Engine(e) => match e {
                crate::core::error::Error::OpenAI(err) => 
                    Errata::new("OpenAI API Error").description(err.to_string()),
                crate::core::error::Error::EmptyResponse(provider) => 
                    Errata::new("Empty Response Error").description(format!("No content received from {}", provider)),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_simple_error() {
        let error = Errata::new("Something went wrong");
        assert_eq!(
            format!("{:?}", error),
            indoc! {"ERROR Something went wrong"}
        );
    }

    #[test]
    fn test_error_with_description() {
        let error = Errata::new("Invalid input")
            .description("Expected a number");
        assert_eq!(
            format!("{:?}", error),
            indoc! {"
                ERROR Invalid input
                Expected a number"}
        );
    }
}
