use std::path::PathBuf;

use nom::branch::alt;
use nom::bytes::complete::take_while1;
use nom::character::complete::{char, space0, space1};
use nom::combinator::{map, opt, recognize};
use nom::multi::many0;
use nom::sequence::{pair, preceded};
use nom::IResult;
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Clone, Serialize, JsonSchema)]
pub struct Prompt {
    pub message: String,
    pub files: Vec<File>,
}

#[derive(Clone, Serialize, JsonSchema)]
pub struct File {
    pub path: String,
    pub content: String,
}

impl Prompt {}

#[derive(Debug, PartialEq)]
pub enum Token {
    Text(String),
    FilePath(PathBuf),
}

impl Prompt {
    // TODO: make `parse` pub(crate)
    pub async fn parse(message: String) -> Result<Prompt, String> {
        let mut prompt = Prompt { message: message.clone(), files: Vec::new() };

        let tokens = match Self::parse_tokens(&message) {
            Ok((_, tokens)) => tokens,
            Err(_) => vec![Token::Text(message)], // Fallback for unparsable input
        };

        for token in tokens {
            if let Token::FilePath(path) = token {
                let content = tokio::fs::read_to_string(&path)
                    .await
                    .map_err(|e| e.to_string())?;
                prompt.add_file(File { path: path.display().to_string(), content });
            }
        }

        Ok(prompt)
    }

    pub fn add_file(&mut self, file: File) {
        self.files.push(file);
    }

    fn parse_tokens(input: &str) -> IResult<&str, Vec<Token>> {
        many0(alt((
            Self::parse_file_path,
            map(Self::parse_word, Token::Text),
        )))(input)
    }

    fn parse_file_path(input: &str) -> IResult<&str, Token> {
        map(
            preceded(char('@'), take_while1(|c: char| !c.is_whitespace())),
            |path: &str| Token::FilePath(PathBuf::from(path)),
        )(input)
    }

    fn parse_word(input: &str) -> IResult<&str, String> {
        let (input, _) = space0(input)?;
        let (input, word) = recognize(pair(
            take_while1(|c: char| !c.is_whitespace() && c != '@'),
            opt(space1),
        ))(input)?;
        Ok((input, word.trim().to_string()))
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_simple_file_path() {
//         let tokens = Prompt::parse_tokens("Check @src/main.rs
// please").unwrap();         assert_eq!(
//             tokens,
//             vec![
//                 Token::Text("Check".to_string()),
//                 Token::FilePath(PathBuf::from("src/main.rs")),
//                 Token::Text("please".to_string()),
//             ]
//         );
//     }

//     #[test]
//     fn test_multiple_file_paths() {
//         let tokens = Prompt::parse_tokens("Compare @file1.rs with
// @file2.rs").unwrap();         assert_eq!(
//             tokens,
//             vec![
//                 Token::Text("Compare".to_string()),
//                 Token::FilePath(PathBuf::from("file1.rs")),
//                 Token::Text("with".to_string()),
//                 Token::FilePath(PathBuf::from("file2.rs")),
//             ]
//         );
//     }

//     #[test]
//     fn test_empty_input() {
//         let tokens = Prompt::parse_tokens("").unwrap();
//         assert_eq!(tokens, Vec::<Token>::new());
//     }

//     #[test]
//     fn test_only_text() {
//         let tokens = Prompt::parse_tokens("just some text").unwrap();
//         assert_eq!(
//             tokens,
//             vec![
//                 Token::Text("just".to_string()),
//                 Token::Text("some".to_string()),
//                 Token::Text("text".to_string()),
//             ]
//         );
//     }
// }
