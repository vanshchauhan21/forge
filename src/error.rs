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

#[derive(Clone)]
struct Errata {
    title: String,
    description: Option<String>,
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
            Error::Engine(e) => Errata {
                title: e.to_string(),
                description: None,
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
        let error = Errata {
            title: "Something went wrong".to_string(),
            description: None,
        };
        assert_eq!(
            format!("{:?}", error),
            indoc! {"ERROR Something went wrong"}
        );
    }

    #[test]
    fn test_error_with_description() {
        let error = Errata {
            title: "Invalid input".to_string(),
            description: Some("Expected a number".to_string()),
        };
        assert_eq!(
            format!("{:?}", error),
            indoc! {"
                ERROR Invalid input
                Expected a number"}
        );
    }
}
