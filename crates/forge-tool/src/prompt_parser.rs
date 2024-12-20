use nom::{
    branch::alt,
    bytes::complete::take_while1,
    character::complete::{char, space0, space1},
    combinator::{map, opt, recognize},
    multi::many0,
    sequence::{pair, preceded},
    IResult,
};
use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub enum Token {
    Text(String),
    FilePath(PathBuf),
}

#[derive(Debug)]
pub struct PromptParser;

impl PromptParser {
    pub fn parse(input: String) -> Vec<Token> {
        match Self::parse_tokens(&input) {
            Ok((_, tokens)) => tokens,
            Err(_) => vec![Token::Text(input)], // Fallback for unparseable input
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_file_path() {
        let tokens = PromptParser::parse("Check @src/main.rs please".to_string());
        assert_eq!(
            tokens,
            vec![
                Token::Text("Check".to_string()),
                Token::FilePath(PathBuf::from("src/main.rs")),
                Token::Text("please".to_string()),
            ]
        );
    }

    #[test]
    fn test_multiple_file_paths() {
        let tokens = PromptParser::parse("Compare @file1.rs with @file2.rs".to_string());
        assert_eq!(
            tokens,
            vec![
                Token::Text("Compare".to_string()),
                Token::FilePath(PathBuf::from("file1.rs")),
                Token::Text("with".to_string()),
                Token::FilePath(PathBuf::from("file2.rs")),
            ]
        );
    }

    #[test]
    fn test_empty_input() {
        let tokens = PromptParser::parse("".to_string());
        assert_eq!(tokens, Vec::<Token>::new());
    }

    #[test]
    fn test_only_text() {
        let tokens = PromptParser::parse("just some text".to_string());
        assert_eq!(
            tokens,
            vec![
                Token::Text("just".to_string()),
                Token::Text("some".to_string()),
                Token::Text("text".to_string()),
            ]
        );
    }
}
