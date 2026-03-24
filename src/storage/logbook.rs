use std::path::PathBuf;
use anyhow::{Context, Result};
use crate::models::LogEntry;

fn logbook_dir() -> Result<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
    Ok(home.join(".alog").join("logbook"))
}

fn entry_path(project: &str, category: &str) -> Result<PathBuf> {
    Ok(logbook_dir()?.join(project).join(format!("{}.json", category)))
}

/// Load entries for a specific project and category.
pub async fn load_entries(project: &str, category: &str) -> Result<Vec<LogEntry>> {
    let path = entry_path(project, category)?;
    if !path.exists() {
        return Ok(vec![]);
    }
    let content = tokio::fs::read_to_string(&path)
        .await
        .with_context(|| format!("Failed to read {}", path.display()))?;
    serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse {}", path.display()))
}

/// Save entries for a specific project and category, creating directories as needed.
pub async fn save_entries(project: &str, category: &str, entries: &[LogEntry]) -> Result<()> {
    let path = entry_path(project, category)?;
    let dir = path.parent().expect("entry path always has a parent");

    tokio::fs::create_dir_all(dir)
        .await
        .with_context(|| format!("Failed to create directory {}", dir.display()))?;

    #[cfg(unix)]
    set_dir_permissions(dir)?;

    let content = serde_json::to_string_pretty(entries)?;
    tokio::fs::write(&path, &content)
        .await
        .with_context(|| format!("Failed to write {}", path.display()))?;

    #[cfg(unix)]
    set_file_permissions(&path)?;

    Ok(())
}

/// Load entries matching optional project and category filters.
/// - `project = None`  → search all project directories
/// - `category = None` → search all category files within the matched project directories
pub async fn load_filtered_entries(
    project: Option<&str>,
    category: Option<&str>,
) -> Result<Vec<LogEntry>> {
    let base = logbook_dir()?;
    if !base.exists() {
        return Ok(vec![]);
    }

    let project_dirs: Vec<PathBuf> = match project {
        Some(p) => vec![base.join(p)],
        None => {
            let mut dirs = Vec::new();
            let mut rd = tokio::fs::read_dir(&base).await?;
            while let Some(entry) = rd.next_entry().await? {
                if entry.file_type().await?.is_dir() {
                    dirs.push(entry.path());
                }
            }
            dirs
        }
    };

    let mut all_entries = Vec::new();

    for dir in project_dirs {
        if !dir.exists() {
            continue;
        }

        let files: Vec<PathBuf> = match category {
            Some(c) => vec![dir.join(format!("{}.json", c))],
            None => {
                let mut files = Vec::new();
                let mut rd = tokio::fs::read_dir(&dir).await?;
                while let Some(entry) = rd.next_entry().await? {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) == Some("json") {
                        files.push(path);
                    }
                }
                files
            }
        };

        for file_path in files {
            if !file_path.exists() {
                continue;
            }
            let content = tokio::fs::read_to_string(&file_path).await?;
            if let Ok(entries) = serde_json::from_str::<Vec<LogEntry>>(&content) {
                all_entries.extend(entries);
            }
        }
    }

    Ok(all_entries)
}

#[cfg(unix)]
fn set_dir_permissions(path: &std::path::Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))
        .with_context(|| format!("Failed to set permissions on {}", path.display()))
}

#[cfg(unix)]
fn set_file_permissions(path: &std::path::Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
        .with_context(|| format!("Failed to set permissions on {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::LogEntry;
    use chrono::Utc;
    use std::env;
    use std::sync::{LazyLock, Mutex};

    /// Tests that mutate the process-wide HOME env var must hold this lock to avoid races.
    static HOME_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    fn make_entry(category: &str, content: &str) -> LogEntry {
        LogEntry {
            id: uuid::Uuid::new_v4().to_string(),
            category: category.into(),
            content: content.into(),
            project: None,
            created_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_save_and_load_roundtrip() {
        let _guard = HOME_LOCK.lock().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        env::set_var("HOME", tmp.path());

        let entries = vec![
            make_entry("notes", "first note"),
            make_entry("notes", "second note"),
        ];
        save_entries("testproject", "notes", &entries).await.unwrap();
        let loaded = load_entries("testproject", "notes").await.unwrap();

        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].content, "first note");
        assert_eq!(loaded[1].content, "second note");
    }

    #[tokio::test]
    async fn test_load_missing_returns_empty() {
        let _guard = HOME_LOCK.lock().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        env::set_var("HOME", tmp.path());

        let result = load_entries("nonexistent", "notes").await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_load_filtered_all_projects() {
        let _guard = HOME_LOCK.lock().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        env::set_var("HOME", tmp.path());

        let a = vec![make_entry("notes", "entry from alpha")];
        let b = vec![make_entry("notes", "entry from beta")];
        save_entries("alpha", "notes", &a).await.unwrap();
        save_entries("beta", "notes", &b).await.unwrap();

        let all = load_filtered_entries(None, Some("notes")).await.unwrap();
        assert_eq!(all.len(), 2);
    }
}
