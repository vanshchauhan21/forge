use std::path::PathBuf;
use std::usize;

use forge_prompt::{PromptData, UserPrompt};
use spinners::Spinner;

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
}

pub struct Loader {
    sp: Spinner,
    title: String,
}

impl Loader {
    pub fn start(title: &str) -> Self {
        println!("│");
        let sp = Spinner::new(spinners::Spinners::Dots, format!(" {}", title));
        Self { sp, title: title.to_string() }
    }

    pub fn stop(self) {
        let title = self.title.clone();
        self.stop_with(title.as_str());
    }
    pub fn stop_with(mut self, text: &str) {
        self.sp.stop();

        let size = termsize::get()
            .map(|u| u.cols as usize)
            .unwrap_or(usize::MAX)
            - 4;
        print!("\r");
        for line in text.lines() {
            if line.len() > size {
                let mut start = 0;
                while start < line.len() {
                    let end = std::cmp::min(start + size, line.len());
                    println!("│  {}", &line[start..end]);
                    start = end;
                }
            } else {
                println!("│  {}", line);
            }
        }
    }
}
