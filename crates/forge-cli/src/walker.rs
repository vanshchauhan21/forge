use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use ignore::WalkBuilder;
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant};

use crate::error::Result;

type Store = HashMap<PathBuf, (Vec<String>, Instant)>;
pub struct Walker {
    cwd: PathBuf,
    cache: Arc<RwLock<Store>>,
}

impl Walker {
    pub fn new(cwd: PathBuf) -> Self {
        Self { cwd, cache: Arc::new(RwLock::new(HashMap::new())) }
    }

    /// Get project files from cache, updating cache asynchronously if needed
    pub async fn get(&self) -> Result<Vec<String>> {
        let path = self.cwd.as_path();
        // Check cache first
        let cache_valid = {
            let cache = self.cache.read().await;
            cache
                .get(path)
                .map(|(_, timestamp)| timestamp.elapsed() < Duration::from_secs(30))
                .unwrap_or(false)
        };

        if cache_valid {
            // Serve from cache
            let cache = self.cache.read().await;
            if let Some((files, _)) = cache.get(path) {
                return Ok(files.clone());
            }
        }

        // Cache miss or expired - get fresh data
        let files = Self::scan_project_files(path)?;

        // Update cache
        {
            let mut cache = self.cache.write().await;
            cache.insert(path.to_path_buf(), (files.clone(), Instant::now()));
        }

        // Spawn background task to keep cache fresh
        let cache = self.cache.clone();
        let path = path.to_path_buf();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(30)).await;
                match Self::scan_project_files(&path) {
                    Ok(new_files) => {
                        let mut cache = cache.write().await;
                        cache.insert(path.clone(), (new_files, Instant::now()));
                    }
                    Err(_) => break, // Stop background task on error
                }
            }
        });

        Ok(files)
    }

    /// Internal function to scan filesystem
    fn scan_project_files(path: &Path) -> std::io::Result<Vec<String>> {
        let mut paths = Vec::new();
        let walker = WalkBuilder::new(path)
            .hidden(true) // Skip hidden files
            .git_global(true) // Use global gitignore
            .git_ignore(true) // Use local .gitignore
            .ignore(true) // Use .ignore files
            .build();

        for entry in walker.flatten() {
            if entry.file_type().is_some_and(|ft| ft.is_file()) {
                paths.push(entry.path().display().to_string());
            }
        }

        Ok(paths)
    }
}
