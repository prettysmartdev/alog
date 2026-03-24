use anyhow::Result;
use crate::models::LogEntry;
use crate::storage::logbook;

/// Handle the `alog write` command.
/// Creates a new log entry and optionally removes an existing one (--replace).
/// Prints the new entry's ID to stdout on success.
pub async fn run(
    category: String,
    entry: String,
    project: Option<String>,
    replace: Option<String>,
) -> Result<()> {
    let project_key = project.as_deref().unwrap_or("global");
    let new_entry = LogEntry::new(category.clone(), entry, project.clone());

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
