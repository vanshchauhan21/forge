use std::path::PathBuf;

use ignore::WalkBuilder;

use crate::Result;

pub struct Walker {
    cwd: PathBuf,
}

impl Walker {
    pub fn new(cwd: PathBuf) -> Self {
        Self { cwd: cwd.clone() }
    }

    /// Internal function to scan filesystem
    pub fn get(&self) -> Result<Vec<String>> {
        let mut files = Vec::new();
        let walk = WalkBuilder::new(&self.cwd)
            .hidden(true) // Skip hidden files
            .git_global(true) // Use global gitignore
            .git_ignore(true) // Use local .gitignore
            .ignore(true) // Use .ignore files
            .build();

        for entry in walk.flatten() {
            let path = entry.path();

            if path.is_file() {
                let path = path.strip_prefix(&self.cwd)?;
                let path = path.to_string_lossy().to_string();
                files.push(path.clone());
            }
        }

        Ok(files)
    }
}
