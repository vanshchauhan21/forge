mod app;
mod error;
mod runtime;

enum Action {
    Initialize,
    ResponseComplete,
}

enum Command {
    Prompt(String),
}

#[derive(Default)]
struct Context {
    files: Vec<String>,
}

#[derive(Default)]
struct State {
    cwd: String,
    context: Context,
}

trait Combine {
    fn combine(self, other: Self) -> Self;
}
