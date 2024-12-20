use serde::Serialize;

use crate::prompt_parser::{PromptParser, Token};

#[derive(Clone, Serialize)]
pub struct Prompt {
    pub message: String,
    pub files: Vec<File>,
}

#[derive(Clone, Serialize)]
pub struct File {
    pub path: String,
    pub content: String,
}

impl Prompt {
    pub async fn parse(message: String) -> Result<Self, String> {
        let mut prompt = Prompt { message: message.clone(), files: Vec::new() };

        let tokens = PromptParser::parse(message);
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
}
