use anyhow::{bail, Result};
use crate::models::LogEntry;
use crate::storage::logbook;

const SESSION_MAX_LEN: usize = 100;

/// Handle the `alog write` command.
/// Creates a new log entry and optionally removes an existing one (--replace).
/// Prints the new entry's ID to stdout on success.
pub async fn run(
    category: String,
    entry: String,
    project: Option<String>,
    replace: Option<String>,
    session: Option<String>,
) -> Result<()> {
    if let Some(ref s) = session {
        if s.len() > SESSION_MAX_LEN {
            bail!("--session value must be 100 characters or fewer (got {})", s.len());
        }
    }

    let project_key = project.as_deref().unwrap_or("global");
    let new_entry = LogEntry::new(category.clone(), entry, project.clone(), session);

    let mut entries = logbook::load_entries(project_key, &category).await?;

    if let Some(ref replace_id) = replace {
        let before = entries.len();
        entries.retain(|e| &e.id != replace_id);
        if entries.len() == before {
            eprintln!("Warning: no entry found with id '{}'", replace_id);
        }
    }

    entries.push(new_entry.clone());
    logbook::save_entries(project_key, &category, &entries).await?;

    println!("{}", new_entry.id);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::sync::{LazyLock, Mutex};

    static HOME_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    #[tokio::test]
    async fn test_session_too_long_is_rejected() {
        let long_session = "x".repeat(101);
        let result = run(
            "notes".into(),
            "content".into(),
            None,
            None,
            Some(long_session),
        )
        .await;
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("100 characters or fewer"));
    }

    #[tokio::test]
    async fn test_session_at_max_length_is_accepted() {
        let _guard = HOME_LOCK.lock().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        env::set_var("HOME", tmp.path());

        let max_session = "s".repeat(100);
        let result = run(
            "notes".into(),
            "content".into(),
            None,
            None,
            Some(max_session),
        )
        .await;
        assert!(result.is_ok());
    }
}
