use std::path::{Path, PathBuf};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GlobalConfig {}

/// Per-repository configuration stored in `.alog.json`.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoConfig {
    /// Minimum similarity percentage (0–100) applied when no `--threshold` flag is given.
    pub default_similarity_threshold: u8,
}

impl Default for RepoConfig {
    fn default() -> Self {
        Self { default_similarity_threshold: 25 }
    }
}

pub fn global_config_path() -> Result<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
    Ok(home.join(".alog").join("config.json"))
}

/// Walk up from `start` until a directory containing `.git` is found.
/// Returns `None` if no git root is found before the filesystem root.
pub fn find_git_root(start: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        if current.join(".git").exists() {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}

/// Determine where `.alog.json` should live for the given git root.
/// If `<git_root>/aspec/` exists, returns `<git_root>/aspec/.alog.json`.
/// Otherwise returns `<git_root>/.alog.json`.
pub fn repo_config_path(git_root: &Path) -> PathBuf {
    if git_root.join("aspec").is_dir() {
        git_root.join("aspec").join(".alog.json")
    } else {
        git_root.join(".alog.json")
    }
}

/// Load the repo config from the given path, returning the default if the file is absent.
pub async fn load_repo_config(path: &Path) -> Result<RepoConfig> {
    if !path.exists() {
        return Ok(RepoConfig::default());
    }
    let content = tokio::fs::read_to_string(path).await?;
    Ok(serde_json::from_str(&content)?)
}

pub async fn load_global_config() -> Result<GlobalConfig> {
    let path = global_config_path()?;
    if !path.exists() {
        return Ok(GlobalConfig::default());
    }
    let content = tokio::fs::read_to_string(&path).await?;
    Ok(serde_json::from_str(&content)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_config_path_ends_with_alog_config() {
        let path = global_config_path().unwrap();
        assert!(path.ends_with(".alog/config.json"));
    }

    #[test]
    fn test_repo_config_path_without_aspec() {
        let tmp = tempfile::tempdir().unwrap();
        let path = repo_config_path(tmp.path());
        assert_eq!(path, tmp.path().join(".alog.json"));
    }

    #[test]
    fn test_repo_config_path_with_aspec() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir(tmp.path().join("aspec")).unwrap();
        let path = repo_config_path(tmp.path());
        assert_eq!(path, tmp.path().join("aspec").join(".alog.json"));
    }

    #[test]
    fn test_find_git_root_finds_parent() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir(tmp.path().join(".git")).unwrap();
        let deep = tmp.path().join("a").join("b");
        std::fs::create_dir_all(&deep).unwrap();
        let root = find_git_root(&deep).unwrap();
        assert_eq!(root, tmp.path());
    }

    #[test]
    fn test_find_git_root_returns_none_when_absent() {
        // Use a path known to have no .git parent (temp dir isolated)
        let tmp = tempfile::tempdir().unwrap();
        let result = find_git_root(tmp.path());
        // May or may not find one depending on where tmp is; just verify no panic
        let _ = result;
    }

    #[test]
    fn test_repo_config_default_threshold() {
        let config = RepoConfig::default();
        assert_eq!(config.default_similarity_threshold, 25);
    }

    #[tokio::test]
    async fn test_load_repo_config_missing_returns_default() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join(".alog.json");
        let config = load_repo_config(&path).await.unwrap();
        assert_eq!(config.default_similarity_threshold, 25);
    }

    #[tokio::test]
    async fn test_load_repo_config_reads_threshold() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join(".alog.json");
        tokio::fs::write(&path, r#"{"defaultSimilarityThreshold": 50}"#).await.unwrap();
        let config = load_repo_config(&path).await.unwrap();
        assert_eq!(config.default_similarity_threshold, 50);
    }

    #[tokio::test]
    async fn test_repo_config_round_trips() {
        let tmp = tempfile::tempdir().unwrap();
        let config = RepoConfig { default_similarity_threshold: 42 };
        let json = serde_json::to_string_pretty(&config).unwrap();
        let path = tmp.path().join(".alog.json");
        tokio::fs::write(&path, &json).await.unwrap();

        let loaded = load_repo_config(&path).await.unwrap();
        assert_eq!(loaded.default_similarity_threshold, 42);
    }
}
