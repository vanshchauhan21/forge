use std::future::Future;
use std::path::PathBuf;

use forge_prompt::{PromptData, UserPrompt};

use crate::Result;

pub struct Tui {
    prompt: UserPrompt,
}

impl Tui {
    pub fn new(cwd: PathBuf) -> Self {
        Self { prompt: UserPrompt::new(cwd) }
    }

    pub async fn ask(&self, prompt: Option<&str>) -> Result<PromptData> {
        println!("│");
        let input = self.prompt.ask(prompt).await?;

        Ok(input)
    }

    pub async fn task<A, F>(&self, title: &str, task: F) -> A
    where
        F: Future<Output = A>,
    {
        println!("│");
        let mut sp = spinners::Spinner::new(spinners::Spinners::Dots, format!(" {}", title));
        let result = task.await;
        sp.stop();
        println!("\r◉  {}", title);

        result
    }

    pub fn item(&self, message: &str) {
        message.lines().for_each(|line| println!("│  {}", line));
    }
}
