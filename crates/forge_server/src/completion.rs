use std::path::PathBuf;

use forge_walker::Walker;
use serde::Serialize;

use crate::Result;

#[derive(Serialize)]
pub struct File {
    pub path: String,
    pub is_dir: bool,
}

pub struct Completion {
    path: String,
}

impl Completion {
    pub fn new(path: impl Into<String>) -> Self {
        Self { path: path.into() }
    }

    pub async fn list(&self) -> Result<Vec<File>> {
        let cwd = PathBuf::from(self.path.clone()); // Use the current working directory
        let walker = Walker::new(cwd);

        let files = walker.get().await?;
        Ok(files
            .into_iter()
            .map(|file| File { path: file.path, is_dir: file.is_dir })
            .collect())
    }
}
