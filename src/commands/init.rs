use std::io::Write;
use anyhow::{Context, Result};
use serde_json::Value;
use crate::storage::config::{find_git_root, repo_config_path, RepoConfig};

/// Base URL for skill files on the project's main branch.
const SKILL_BASE_URL: &str =
    "https://raw.githubusercontent.com/prettysmartdev/alog/refs/heads/main/.claude/skills/";

/// Skill names installed by `alog init`. Each becomes a directory containing `SKILL.md`.
const SKILL_NAMES: &[&str] = &["alog", "alog-summarize", "alog-export"];

/// Extra files (besides SKILL.md) to download for specific skills.
/// Each entry is (skill_name, &[filename]).
const SKILL_EXTRA_FILES: &[(&str, &[&str])] = &[
    ("alog", &["get-project.sh"]),
];

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
            // Download any extra files (e.g. helper scripts) for this skill.
            for (name, files) in SKILL_EXTRA_FILES {
                if *name == *skill_name {
                    for file in *files {
                        install_skill_file(&git_root, skill_name, file).await?;
                    }
                }
            }
        }
    }

    // Offer to patch Dockerfile.dev if one exists in the git root.
    let dockerfile_path = git_root.join("Dockerfile.dev");
    if dockerfile_path.exists() {
        if prompt_yes_no("Dockerfile.dev detected — add alog installation via the install script? [y/N] ")? {
            patch_dockerfile(&dockerfile_path).await?;
        }
    }

    // Offer to add alog permissions to Claude Code project settings.
    if prompt_yes_no("Allow Claude Code to run alog commands without asking permission? [y/N] ")? {
        patch_claude_settings(&git_root).await?;
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
    install_skill_file(git_root, skill_name, "SKILL.md").await
}

/// Download a single file for a skill into `.claude/skills/<name>/`.
///
/// For shell scripts the file is made executable (0o755) after writing; all
/// other files are written with the standard restricted permissions (0o600).
async fn install_skill_file(git_root: &std::path::Path, skill_name: &str, filename: &str) -> Result<()> {
    let skill_dir = git_root.join(".claude").join("skills").join(skill_name);
    tokio::fs::create_dir_all(&skill_dir)
        .await
        .with_context(|| format!("Failed to create directory {}", skill_dir.display()))?;

    let url = format!("{}{}/{}", SKILL_BASE_URL, skill_name, filename);
    let response = reqwest::get(&url)
        .await
        .with_context(|| format!("Failed to download skill file from {}", url))?;

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

    let skill_path = skill_dir.join(filename);
    tokio::fs::write(&skill_path, &content)
        .await
        .with_context(|| format!("Failed to write {}", skill_path.display()))?;

    #[cfg(unix)]
    if filename.ends_with(".sh") {
        set_executable_permissions(&skill_path)?;
    } else {
        set_file_permissions(&skill_path)?;
    }

    println!("Installed {}", skill_path.display());
    Ok(())
}

#[cfg(unix)]
fn set_file_permissions(path: &std::path::Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
        .with_context(|| format!("Failed to set permissions on {}", path.display()))
}

#[cfg(unix)]
fn set_executable_permissions(path: &std::path::Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))
        .with_context(|| format!("Failed to set permissions on {}", path.display()))
}

/// Returns the Dockerfile snippet that installs alog via the official install script.
///
/// `ALOG_INSTALL_ACCEPT_DEFAULTS=1` suppresses interactive prompts so the script
/// runs unattended during a container build.
fn alog_dockerfile_snippet() -> String {
    "\n# Install alog\n\
     ENV ALOG_INSTALL_ACCEPT_DEFAULTS=1\n\
     RUN curl -s https://prettysmart.dev/install/alog.sh | sh\n"
        .to_string()
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

/// The set of Bash permission rules added to the Claude Code settings file.
const ALOG_PERMISSIONS: &[&str] = &[
    "Bash(alog write:*)",
    "Bash(alog recall:*)",
    "Bash(alog export:*)",
    "Bash(.claude/skills/alog/get-project.sh:*)",
];

/// Patches `.claude/settings.json` in the git root to allow alog commands.
///
/// Reads the existing settings (creating a minimal skeleton if absent), merges the
/// alog permission rules into `permissions.allow`, and writes the result back.
/// Only entries that are not already present are added, so the function is idempotent.
async fn patch_claude_settings(git_root: &std::path::Path) -> Result<()> {
    let settings_dir = git_root.join(".claude");
    tokio::fs::create_dir_all(&settings_dir)
        .await
        .with_context(|| format!("Failed to create directory {}", settings_dir.display()))?;

    let settings_path = settings_dir.join("settings.json");

    // Load existing settings or start with an empty object.
    let mut root: Value = if settings_path.exists() {
        let raw = tokio::fs::read_to_string(&settings_path)
            .await
            .with_context(|| format!("Failed to read {}", settings_path.display()))?;
        serde_json::from_str(&raw)
            .with_context(|| format!("Failed to parse JSON in {}", settings_path.display()))?
    } else {
        serde_json::json!({})
    };

    // Ensure permissions.allow exists and is an array.
    let allow = root
        .pointer_mut("/permissions/allow")
        .and_then(|v| v.as_array_mut());

    if allow.is_none() {
        // Build the nested structure if absent.
        root["permissions"] = serde_json::json!({ "allow": [] });
    }

    // Now we can safely get the mutable array.
    let allow = root["permissions"]["allow"]
        .as_array_mut()
        .expect("permissions.allow is always an array at this point");

    let mut added = 0usize;
    for &rule in ALOG_PERMISSIONS {
        let rule_value = Value::String(rule.to_string());
        if !allow.contains(&rule_value) {
            allow.push(rule_value);
            added += 1;
        }
    }

    let json = serde_json::to_string_pretty(&root)?;
    tokio::fs::write(&settings_path, &json)
        .await
        .with_context(|| format!("Failed to write {}", settings_path.display()))?;

    #[cfg(unix)]
    set_file_permissions(&settings_path)?;

    if added > 0 {
        println!("Patched {} (+{} alog permission rules)", settings_path.display(), added);
    } else {
        println!("alog permissions already present in {}", settings_path.display());
    }

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
    fn test_alog_dockerfile_snippet_uses_install_script() {
        let snippet = alog_dockerfile_snippet();
        assert!(snippet.contains("curl"), "snippet must use curl");
        assert!(
            snippet.contains("prettysmart.dev/install/alog.sh"),
            "snippet must use the official install script URL"
        );
        assert!(
            snippet.contains("ALOG_INSTALL_ACCEPT_DEFAULTS=1"),
            "snippet must set ALOG_INSTALL_ACCEPT_DEFAULTS=1 for unattended installs"
        );
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
        assert!(result.contains("Install alog"));
    }

    #[tokio::test]
    async fn test_patch_dockerfile_inserts_before_cmd() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("Dockerfile.dev");
        let original = "FROM ubuntu:22.04\nRUN apt-get update\nCMD [\"/bin/bash\"]\n";
        tokio::fs::write(&path, original).await.unwrap();

        patch_dockerfile(&path).await.unwrap();

        let result = tokio::fs::read_to_string(&path).await.unwrap();
        let alog_pos = result.find("Install alog").unwrap();
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
        let alog_pos = result.find("Install alog").unwrap();
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

    #[tokio::test]
    async fn test_patch_claude_settings_creates_settings_when_absent() {
        let tmp = tempfile::tempdir().unwrap();

        patch_claude_settings(tmp.path()).await.unwrap();

        let settings_path = tmp.path().join(".claude").join("settings.json");
        assert!(settings_path.exists(), "settings.json should be created");

        let raw = tokio::fs::read_to_string(&settings_path).await.unwrap();
        let value: serde_json::Value = serde_json::from_str(&raw).unwrap();
        let allow = value["permissions"]["allow"].as_array().unwrap();

        for rule in ALOG_PERMISSIONS {
            assert!(
                allow.contains(&serde_json::Value::String(rule.to_string())),
                "missing rule: {}",
                rule
            );
        }
    }

    #[tokio::test]
    async fn test_patch_claude_settings_merges_into_existing_file() {
        let tmp = tempfile::tempdir().unwrap();
        let settings_dir = tmp.path().join(".claude");
        tokio::fs::create_dir_all(&settings_dir).await.unwrap();
        let settings_path = settings_dir.join("settings.json");

        // Write a pre-existing settings file with one existing permission.
        let existing = serde_json::json!({
            "permissions": {
                "allow": ["Bash(cargo test:*)"]
            }
        });
        tokio::fs::write(&settings_path, serde_json::to_string_pretty(&existing).unwrap())
            .await
            .unwrap();

        patch_claude_settings(tmp.path()).await.unwrap();

        let raw = tokio::fs::read_to_string(&settings_path).await.unwrap();
        let value: serde_json::Value = serde_json::from_str(&raw).unwrap();
        let allow = value["permissions"]["allow"].as_array().unwrap();

        // Original rule preserved.
        assert!(allow.contains(&serde_json::Value::String("Bash(cargo test:*)".to_string())));
        // alog rules added.
        for rule in ALOG_PERMISSIONS {
            assert!(
                allow.contains(&serde_json::Value::String(rule.to_string())),
                "missing rule: {}",
                rule
            );
        }
    }

    #[tokio::test]
    async fn test_patch_claude_settings_is_idempotent() {
        let tmp = tempfile::tempdir().unwrap();

        patch_claude_settings(tmp.path()).await.unwrap();
        patch_claude_settings(tmp.path()).await.unwrap(); // run twice

        let settings_path = tmp.path().join(".claude").join("settings.json");
        let raw = tokio::fs::read_to_string(&settings_path).await.unwrap();
        let value: serde_json::Value = serde_json::from_str(&raw).unwrap();
        let allow = value["permissions"]["allow"].as_array().unwrap();

        // No duplicates: each rule appears exactly once.
        for rule in ALOG_PERMISSIONS {
            let count = allow
                .iter()
                .filter(|v| v.as_str() == Some(rule))
                .count();
            assert_eq!(count, 1, "rule '{}' should appear exactly once", rule);
        }
    }
}
