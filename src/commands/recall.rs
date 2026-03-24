use anyhow::Result;
use crate::search;
use crate::storage::{config, logbook};

/// Handle the `alog recall` command.
/// Fuzzy-searches log entries and prints results to stdout, ranked by similarity.
///
/// The effective similarity threshold is resolved in this order:
/// 1. `--threshold` flag (explicit, always wins)
/// 2. `defaultSimilarityThreshold` from `.alog.json` in the current git root (or its `aspec/` subdirectory)
/// 3. 0 (no filtering)
pub async fn run(
    category: String,
    search_term: String,
    project: Option<String>,
    count: Option<usize>,
    threshold: Option<u8>,
) -> Result<()> {
    let project_filter = project.as_deref();
    let category_filter = if category == "all" { None } else { Some(category.as_str()) };

    let entries = logbook::load_filtered_entries(project_filter, category_filter).await?;

    let effective_threshold = resolve_threshold(threshold).await;
    let mut results = search::fuzzy_search(&entries, &search_term, effective_threshold);

    if let Some(n) = count {
        results.truncate(n);
    }

    for (score, entry) in &results {
        println!(
            "[{}] ({:.0}%) [{}] {}",
            entry.id,
            score * 100.0,
            entry.category,
            entry.content
        );
    }

    Ok(())
}

/// Determine the threshold to use (as a 0.0–1.0 value).
/// Explicit `--threshold` overrides repo config; repo config overrides the default of 0.
async fn resolve_threshold(flag: Option<u8>) -> f64 {
    if let Some(t) = flag {
        return t as f64 / 100.0;
    }

    if let Ok(cwd) = std::env::current_dir() {
        if let Some(git_root) = config::find_git_root(&cwd) {
            let config_path = config::repo_config_path(&git_root);
            if let Ok(repo_cfg) = config::load_repo_config(&config_path).await {
                return repo_cfg.default_similarity_threshold as f64 / 100.0;
            }
        }
    }

    0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_resolve_threshold_uses_flag_when_provided() {
        // The flag always wins regardless of any repo config
        let result = resolve_threshold(Some(75)).await;
        assert!((result - 0.75).abs() < 1e-10);
    }

    #[tokio::test]
    async fn test_resolve_threshold_zero_flag() {
        let result = resolve_threshold(Some(0)).await;
        assert!((result - 0.0).abs() < 1e-10);
    }
}
