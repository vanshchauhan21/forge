
use crate::error::Result;
use axum::{
    body::Body,
    http::{Request, Response},
};
use derive_more::Debug;

#[derive(Clone)]
pub struct Exec {}

impl Exec {
    pub fn new() -> Self {
        Exec {}
    }

    pub async fn execute(&self, request: Request<Body>) -> Result<Response<Body>> {
        println!("{:?}", request);
        todo!()
    }
}

struct LLMAgent {}

#[derive(Debug)]
enum Prompt {
    Message(String),
}

impl LLMAgent {
    async fn execute(prompt: Prompt) {
        use rig::{completion::Prompt, providers::openai};

        // Create OpenAI client and model
        // This requires the `OPENAI_API_KEY` environment variable to be set.
        let gpt4 = openai::Client::from_env().agent("gpt-4").build();

        // Prompt the model and print its response
        let response = gpt4
            .prompt(format!("{:?}", prompt).as_str())
            .await
            .expect("Failed to prompt GPT-4");

        println!("GPT-4: {response}");
    }
}
