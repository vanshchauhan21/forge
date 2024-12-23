use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CompletionType {
    Command,
    File,
    Directory,
    Variable,
}

#[derive(Serialize)]
pub struct Completion {
    pub text: String,
    pub completion_type: CompletionType,
}

pub async fn get_completions() -> Vec<Completion> {
    // For now returning sample completions
    // This can be expanded to fetch from actual completion source
    vec![
        Completion {
            text: "git status".to_string(),
            completion_type: CompletionType::Command,
        },
        Completion {
            text: "README.md".to_string(),
            completion_type: CompletionType::File,
        },
        Completion {
            text: "src/".to_string(),
            completion_type: CompletionType::Directory,
        },
        Completion {
            text: "$HOME".to_string(),
            completion_type: CompletionType::Variable,
        },
    ]
}
