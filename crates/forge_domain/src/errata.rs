use std::fmt::{Debug, Display, Formatter};

use derive_setters::Setters;
use serde::Serialize;

#[derive(Clone, Setters, Serialize, PartialEq, Eq)]
pub struct Errata {
    pub message: String,
    #[setters(strip_option)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<u32>,
    #[setters(strip_option, into)]
    pub description: Option<String>,
}

impl Errata {
    pub fn new(title: impl Into<String>) -> Self {
        Self { message: title.into(), description: None, code: None }
    }
}

impl Display for Errata {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Debug for Errata {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(desc) = &self.description {
            if !desc.trim().is_empty() {
                write!(f, "\n{}", desc)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::Errata;

    #[test]
    fn test_simple_error() {
        let error = Errata::new("Something went wrong");
        assert_eq!(format!("{:?}", error), "Something went wrong");
    }

    #[test]
    fn test_error_with_description() {
        let error = Errata::new("Invalid input").description("Expected a number");
        assert_eq!(format!("{:?}", error), "Invalid input\nExpected a number");
    }
}
