use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Git command failed: {0}")]
    Git(String),

    #[error("IO error: {0}")]
    IO(#[from] io::Error),

    #[error("Invalid worktree name '{name}': {reason}")]
    WorktreeName { name: String, reason: &'static str },

    #[error("Worktree creation failed: {0}")]
    WorktreeCreate(String),
}

pub struct WorkTree {
    path: PathBuf,
}

impl WorkTree {
    /// Creates a new WorkTree instance with the specified name
    pub fn new(name: &str) -> Result<Self, Error> {
        if !Self::is_valid_name(name) {
            return Err(Error::WorktreeName {
                name: name.to_string(),
                reason: "Use only alphanumeric characters, hyphens, and underscores",
            });
        }
        let path = Self::create(name)?;
        Ok(Self { path })
    }

    /// Validates the worktree name
    fn is_valid_name(name: &str) -> bool {
        if name.is_empty() {
            return false;
        }
        name.chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    }

    /// Creates a new git worktree with the specified name
    fn create(worktree_name: &str) -> Result<PathBuf, Error> {
        let current_dir = std::env::current_dir()?;

        // Get current commit hash
        let hash_output = Command::new("git")
            .arg("-c")
            .arg("core.pager=cat")
            .args(["rev-parse", "HEAD"])
            .current_dir(&current_dir)
            .output()?;

        if !hash_output.status.success() {
            let stderr = String::from_utf8_lossy(&hash_output.stderr);
            let stdout = String::from_utf8_lossy(&hash_output.stdout);
            return Err(Error::Git(format!(
                "Failed to get current commit hash\nstdout: {}\nstderr: {}",
                stdout, stderr
            )));
        }

        let commit_hash = String::from_utf8_lossy(&hash_output.stdout)
            .trim()
            .to_string();

        // Create the worktree using the commit hash
        let output = Command::new("git")
            .arg("-c")
            .arg("core.pager=cat")
            .args([
                "worktree",
                "add",
                "-b",
                worktree_name,
                worktree_name,
                &commit_hash,
            ])
            .current_dir(&current_dir)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(Error::WorktreeCreate(format!(
                "Failed to create worktree\nstdout: {}\nstderr: {}",
                stdout, stderr
            )));
        }

        let path = current_dir.join(worktree_name);

        if !path.exists() {
            return Err(Error::WorktreeCreate(format!(
                "Worktree directory was not created at {}",
                path.display()
            )));
        }

        // Get the absolute path
        let path = path.canonicalize()?;

        Ok(path)
    }

    /// Returns the path to the worktree
    pub fn get_path(&self) -> &Path {
        &self.path
    }

    /// Changes the current working directory to the worktree
    pub fn change_dir(&self) -> Result<(), Error> {
        std::env::set_current_dir(&self.path)?;
        Ok(())
    }
}
