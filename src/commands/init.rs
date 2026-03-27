use std::io::Write;
use anyhow::{Context, Result};
use crate::storage::config::{find_git_root, repo_config_path, RepoConfig};

/// Base URL for skill files on the project's main branch.
const SKILL_BASE_URL: &str =
    "https://raw.githubusercontent.com/prettysmartdev/alog/refs/heads/main/.claude/skills/";

/// Skill names installed by `alog init`. Each becomes a directory containing `SKILL.md`.
const SKILL_NAMES: &[&str] = &["alog", "alog-summarize", "alog-export"];

/// GitHub repository slug used for release asset URLs.
const GITHUB_REPO: &str = "prettysmartdev/alog";

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

    // Ask the user whether to install the Claude Code skills.
    if prompt_yes_no("Download Claude Code skills (.claude/skills/) from the alog repository? [y/N] ")? {
        for skill_name in SKILL_NAMES {
            install_skill(&git_root, skill_name).await?;
        }
    }

    // Offer to patch Dockerfile.dev if one exists in the git root.
    let dockerfile_path = git_root.join("Dockerfile.dev");
    if dockerfile_path.exists() {
        if prompt_yes_no("Dockerfile.dev detected — add alog installation from the latest GitHub release? [y/N] ")? {
            patch_dockerfile(&dockerfile_path).await?;
        }
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

/// Install a single skill by downloading its `SKILL.md` into `.claude/skills/<name>/`.
///
/// Claude Code loads skills from directories containing a `SKILL.md` file, not flat `.md` files.
async fn install_skill(git_root: &std::path::Path, skill_name: &str) -> Result<()> {
    let skill_dir = git_root.join(".claude").join("skills").join(skill_name);
    tokio::fs::create_dir_all(&skill_dir)
        .await
        .with_context(|| format!("Failed to create directory {}", skill_dir.display()))?;

    let url = format!("{}{}/SKILL.md", SKILL_BASE_URL, skill_name);
    let response = reqwest::get(&url)
        .await
        .with_context(|| format!("Failed to download skill from {}", url))?;

    if !response.status().is_success() {
        anyhow::bail!(
            "Download failed with status {}: {}",
            response.status(),
            url
        );
    }

    let content = response
        .text()
        .await
        .context("Failed to read skill file content")?;

    let skill_path = skill_dir.join("SKILL.md");
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

/// Returns the Dockerfile snippet that installs alog from the latest GitHub release.
fn alog_dockerfile_snippet() -> String {
    format!(
        "\n# Install alog from latest GitHub release\n\
         RUN ALOG_VERSION=$(curl -fsSL https://api.github.com/repos/{repo}/releases/latest \\\n\
         \t\t| grep '\"tag_name\"' \\\n\
         \t\t| sed 's/.*\"tag_name\": *\"\\([^\"]*\\)\".*/\\1/') \\\n\
         \t&& curl -fsSL \"https://github.com/{repo}/releases/download/${{ALOG_VERSION}}/alog-linux-amd64.tar.gz\" \\\n\
         \t\t-o /tmp/alog.tar.gz \\\n\
         \t&& tar -xzf /tmp/alog.tar.gz -C /usr/local/bin/ \\\n\
         \t&& rm /tmp/alog.tar.gz\n",
        repo = GITHUB_REPO
    )
}

/// Patches `Dockerfile.dev` to include an `alog` installation block.
///
/// The snippet is inserted immediately before the first `CMD` or `ENTRYPOINT` instruction.
/// If neither is present the snippet is appended at the end of the file.
async fn patch_dockerfile(path: &std::path::Path) -> Result<()> {
    let content = tokio::fs::read_to_string(path)
        .await
        .with_context(|| format!("Failed to read {}", path.display()))?;

    let snippet = alog_dockerfile_snippet();

    // Find the byte offset of the first CMD or ENTRYPOINT line so the snippet
    // lands before the container entry point.
    let insert_pos = content
        .lines()
        .enumerate()
        .find(|(_, line)| {
            let trimmed = line.trim_start();
            trimmed.starts_with("CMD") || trimmed.starts_with("ENTRYPOINT")
        })
        .map(|(idx, _)| {
            // byte offset of that line
            content
                .lines()
                .take(idx)
                .map(|l| l.len() + 1) // +1 for '\n'
                .sum::<usize>()
        });

    let new_content = match insert_pos {
        Some(pos) => {
            let (before, after) = content.split_at(pos);
            format!("{}{}{}", before, snippet, after)
        }
        None => format!("{}{}", content, snippet),
    };

    tokio::fs::write(path, &new_content)
        .await
        .with_context(|| format!("Failed to write {}", path.display()))?;

    #[cfg(unix)]
    set_file_permissions(path)?;

    println!("Patched {}", path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_base_url_points_to_prettysmart_repo() {
        assert!(
            SKILL_BASE_URL.contains("prettysmartdev/alog"),
            "SKILL_BASE_URL must point to prettysmartdev/alog, got: {}",
            SKILL_BASE_URL
        );
    }

    #[test]
    fn test_skill_names_includes_all_three_skills() {
        assert!(SKILL_NAMES.contains(&"alog"));
        assert!(SKILL_NAMES.contains(&"alog-summarize"));
        assert!(SKILL_NAMES.contains(&"alog-export"));
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

    #[test]
    fn test_alog_dockerfile_snippet_contains_github_repo() {
        let snippet = alog_dockerfile_snippet();
        assert!(snippet.contains(GITHUB_REPO));
        assert!(snippet.contains("alog-linux-amd64.tar.gz"));
        assert!(snippet.contains("curl"));
        assert!(snippet.contains("/usr/local/bin/"));
    }

    #[tokio::test]
    async fn test_patch_dockerfile_appends_when_no_cmd() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("Dockerfile.dev");
        let original = "FROM ubuntu:22.04\nRUN apt-get update\n";
        tokio::fs::write(&path, original).await.unwrap();

        patch_dockerfile(&path).await.unwrap();

        let result = tokio::fs::read_to_string(&path).await.unwrap();
        assert!(result.starts_with(original));
        assert!(result.contains("alog-linux-amd64.tar.gz"));
    }

    #[tokio::test]
    async fn test_patch_dockerfile_inserts_before_cmd() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("Dockerfile.dev");
        let original = "FROM ubuntu:22.04\nRUN apt-get update\nCMD [\"/bin/bash\"]\n";
        tokio::fs::write(&path, original).await.unwrap();

        patch_dockerfile(&path).await.unwrap();

        let result = tokio::fs::read_to_string(&path).await.unwrap();
        let alog_pos = result.find("alog-linux-amd64.tar.gz").unwrap();
        let cmd_pos = result.find("CMD [\"/bin/bash\"]").unwrap();
        assert!(alog_pos < cmd_pos, "alog snippet should appear before CMD");
    }

    #[tokio::test]
    async fn test_patch_dockerfile_inserts_before_entrypoint() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("Dockerfile.dev");
        let original = "FROM ubuntu:22.04\nENTRYPOINT [\"/start.sh\"]\n";
        tokio::fs::write(&path, original).await.unwrap();

        patch_dockerfile(&path).await.unwrap();

        let result = tokio::fs::read_to_string(&path).await.unwrap();
        let alog_pos = result.find("alog-linux-amd64.tar.gz").unwrap();
        let ep_pos = result.find("ENTRYPOINT").unwrap();
        assert!(alog_pos < ep_pos, "alog snippet should appear before ENTRYPOINT");
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
