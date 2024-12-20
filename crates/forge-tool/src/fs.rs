use crate::ToolTrait;

pub(crate) struct FSRead;
pub(crate) struct FSSearch;
pub(crate) struct FSList;
pub(crate) struct FSFileInfo;

#[async_trait::async_trait]
impl ToolTrait for FSRead {
    type Input = String;
    type Output = String;

    fn description(&self) -> String {
        "Read a file".to_string()
    }

    async fn call(&self, input: Self::Input) -> Result<Self::Output, String> {
        let content = tokio::fs::read_to_string(&input)
            .await
            .map_err(|e| e.to_string())?;
        Ok(content)
    }
}

#[async_trait::async_trait]
impl ToolTrait for FSSearch {
    type Input = (String, String);
    type Output = Vec<String>;

    fn description(&self) -> String {
        "Search for files and directories recursively in a given directory. Input is (directory_path, search_pattern) where search_pattern is matched against file/directory names".to_string()
    }

    async fn call(&self, input: Self::Input) -> Result<Self::Output, String> {
        let (dir, pattern) = input;
        let pattern = pattern.to_lowercase();

        async fn search(dir: &std::path::Path, pattern: &str) -> Result<Vec<String>, String> {
            let mut matches = Vec::new();
            let mut walker = tokio::fs::read_dir(dir).await.map_err(|e| e.to_string())?;

            while let Some(entry) = walker.next_entry().await.map_err(|e| e.to_string())? {
                let path = entry.path();
                if let Some(name) = path.file_name() {
                    let name = name.to_string_lossy().to_lowercase();
                    if name.contains(pattern) {
                        matches.push(path.to_string_lossy().to_string());
                    }
                }

                if path.is_dir() {
                    matches.extend(Box::pin(search(&path, pattern)).await?);
                }
            }
            Ok(matches)
        }

        Ok(Box::pin(search(std::path::Path::new(&dir), &pattern)).await?)
    }
}

#[async_trait::async_trait]
impl ToolTrait for FSList {
    type Input = String;
    type Output = Vec<String>;

    fn description(&self) -> String {
        "List files and directories in a given directory".to_string()
    }

    async fn call(&self, input: Self::Input) -> Result<Self::Output, String> {
        let dir = std::path::Path::new(&input);
        let mut paths = Vec::new();
        let mut walker = tokio::fs::read_dir(dir).await.map_err(|e| e.to_string())?;

        while let Ok(f) = walker.next_entry().await {
            if let Some(entry) = f {
                let file_type = entry.file_type().await.map_err(|e| e.to_string())?;
                let prefix = if file_type.is_dir() {
                    "[DIR]"
                } else {
                    "[FILE]"
                };
                paths.push(format!("{} {}", prefix, entry.path().display()));
            }
        }
        Ok(paths)
    }
}

#[async_trait::async_trait]
impl ToolTrait for FSFileInfo {
    type Input = String;
    type Output = String;

    fn description(&self) -> String {
        "Get information about a file or directory".to_string()
    }

    async fn call(&self, input: Self::Input) -> Result<Self::Output, String> {
        let meta = tokio::fs::metadata(input)
            .await
            .map_err(|e| e.to_string())?;
        Ok(format!("{:?}", meta))
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_id(){
        assert!(FSRead.id().0.ends_with("fs/fs_read"));
        assert!(FSSearch.id().0.ends_with("fs/fs_search"));
        assert!(FSList.id().0.ends_with("fs/fs_list"));
        assert!(FSFileInfo.id().0.ends_with("fs/fs_file_info"));
    }
}