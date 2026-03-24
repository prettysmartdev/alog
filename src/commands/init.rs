use std::io::Write;
use anyhow::{Context, Result};
use crate::storage::config::{find_git_root, repo_config_path, RepoConfig};

/// URL of the alog skill file on the project's main branch.
const SKILL_URL: &str =
    "https://raw.githubusercontent.com/anthropics/alog/main/.claude/skills/alog.md";

/// Handle the `alog init` command.
///
/// Creates a `.alog.json` repo config (defaultSimilarityThreshold = 25) at the
/// appropriate location within the current git root, then optionally downloads
/// the alog Claude Code skill file after asking the user for permission.
pub async fn run() -> Result<()> {
    let cwd = std::env::current_dir().context("Could not determine current directory")?;

    let git_root = find_git_root(&cwd)
        .ok_or_else(|| anyhow::anyhow!("Not inside a git repository — run `git init` first"))?;

    let config_path = repo_config_path(&git_root);

    let config = RepoConfig::default();
    let json = serde_json::to_string_pretty(&config)?;

    if let Some(parent) = config_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .with_context(|| format!("Failed to create directory {}", parent.display()))?;
    }

    tokio::fs::write(&config_path, &json)
        .await
        .with_context(|| format!("Failed to write {}", config_path.display()))?;

    #[cfg(unix)]
    set_file_permissions(&config_path)?;

    println!("Created {}", config_path.display());

    // Ask the user whether to install the Claude Code skill.
    if prompt_yes_no("Download .claude/skills/alog.md from the alog repository? [y/N] ")? {
        install_skill(&git_root).await?;
    }

    Ok(())
}

fn prompt_yes_no(prompt: &str) -> Result<bool> {
    print!("{}", prompt);
    std::io::stdout().flush()?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    Ok(matches!(input.trim().to_lowercase().as_str(), "y" | "yes"))
}

async fn install_skill(git_root: &std::path::Path) -> Result<()> {
    let skill_dir = git_root.join(".claude").join("skills");
    tokio::fs::create_dir_all(&skill_dir)
        .await
        .with_context(|| format!("Failed to create directory {}", skill_dir.display()))?;

    let response = reqwest::get(SKILL_URL)
        .await
        .with_context(|| format!("Failed to download skill from {}", SKILL_URL))?;

    if !response.status().is_success() {
        anyhow::bail!(
            "Download failed with status {}: {}",
            response.status(),
            SKILL_URL
        );
    }

    let content = response
        .text()
        .await
        .context("Failed to read skill file content")?;

    let skill_path = skill_dir.join("alog.md");
    tokio::fs::write(&skill_path, &content)
        .await
        .with_context(|| format!("Failed to write {}", skill_path.display()))?;

    #[cfg(unix)]
    set_file_permissions(&skill_path)?;

    println!("Installed {}", skill_path.display());
    Ok(())
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

    #[test]
    fn test_skill_url_is_non_empty() {
        assert!(!SKILL_URL.is_empty());
    }

    #[tokio::test]
    async fn test_init_creates_config_without_aspec() {
        let tmp = tempfile::tempdir().unwrap();
        // Create a fake .git directory so find_git_root can find it
        std::fs::create_dir(tmp.path().join(".git")).unwrap();

        // Override CWD by working directly with paths
        let config_path = repo_config_path(tmp.path());
        assert_eq!(config_path, tmp.path().join(".alog.json"));

        let config = RepoConfig::default();
        let json = serde_json::to_string_pretty(&config).unwrap();
        tokio::fs::write(&config_path, &json).await.unwrap();

        let content = tokio::fs::read_to_string(&config_path).await.unwrap();
        let loaded: RepoConfig = serde_json::from_str(&content).unwrap();
        assert_eq!(loaded.default_similarity_threshold, 25);
    }

    #[tokio::test]
    async fn test_init_creates_config_inside_aspec() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir(tmp.path().join(".git")).unwrap();
        std::fs::create_dir(tmp.path().join("aspec")).unwrap();

        let config_path = repo_config_path(tmp.path());
        assert_eq!(config_path, tmp.path().join("aspec").join(".alog.json"));

        let config = RepoConfig::default();
        let json = serde_json::to_string_pretty(&config).unwrap();
        if let Some(parent) = config_path.parent() {
            tokio::fs::create_dir_all(parent).await.unwrap();
        }
        tokio::fs::write(&config_path, &json).await.unwrap();

        let content = tokio::fs::read_to_string(&config_path).await.unwrap();
        let loaded: RepoConfig = serde_json::from_str(&content).unwrap();
        assert_eq!(loaded.default_similarity_threshold, 25);
    }
}
