use inquire::{autocompletion::Replacement, Autocomplete};

#[derive(Clone)]
pub struct Completion {
    suggestions: Vec<String>,
}

impl Completion {
    pub fn new(completions: Vec<impl ToString>) -> Self {
        Self::from_iter(completions.into_iter())
    }

    pub fn from_iter<T: ToString>(iter: impl Iterator<Item = T>) -> Self {
        Self {
            suggestions: iter.map(|a| a.to_string()).collect(),
        }
    }
}

impl Autocomplete for Completion {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, inquire::CustomUserError> {
        // Performs a case-insensitive substring search on the suggestions.
        let input = input.trim().to_lowercase();
        let suggestions = if input.is_empty() {
            Vec::new()
        } else {
            self.suggestions
                .iter()
                .filter(|c| c.to_lowercase().starts_with(&input))
                .cloned()
                .collect()
        };

        Ok(suggestions)
    }

    fn get_completion(
        &mut self,
        _: &str,
        highlighted_suggestion: Option<String>,
    ) -> Result<inquire::autocompletion::Replacement, inquire::CustomUserError> {
        Ok(Replacement::from(highlighted_suggestion))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn test_completion() {
        let mut completion = Completion::new(vec!["/ASK", "/EDIT", "/QUIT"]);
        let actual = completion.get_suggestions("").unwrap();
        let expected: Vec<&str> = Vec::new();
        assert_eq!(actual, expected);

        let actual = completion.get_suggestions("/").unwrap();
        let expected = vec!["/ASK", "/EDIT", "/QUIT"];
        assert_eq!(actual, expected);

        let actual = completion.get_suggestions("/a").unwrap();
        let expected = vec!["/ASK"];
        assert_eq!(actual, expected);
    }
}
