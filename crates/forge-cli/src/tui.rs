use crate::Result;
use forge_prompt::{ResolvePrompt, UserPrompt};
use std::{future::Future, path::PathBuf};

pub struct Tui {
    prompt: UserPrompt,
}

impl Tui {
    pub fn new(cwd: PathBuf) -> Self {
        Self { prompt: UserPrompt::new(cwd) }
    }

    pub async fn ask(&self, prompt: Option<&str>) -> Result<ResolvePrompt> {
        println!("│");
        Ok(self.prompt.ask(prompt).await?)
    }

    pub async fn task<A, F>(&self, message: &str, task: F) -> A
    where
        F: Future<Output = A>,
    {
        println!("│");

        let mut sp = spinners::Spinner::new(spinners::Spinners::Dots, format!(" {}", message));
        let result = task.await;
        sp.stop();
        println!("\r◉  {}", message);

        result
    }
}
