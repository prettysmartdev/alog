use anyhow::Result;
use std::fmt::Write as FmtWrite;
use crate::storage::logbook;

/// Handle the `alog export` command.
/// Loads entries matching the given filters and writes them as Markdown
/// to a file path or stdout (when `output` is `"-"`).
pub async fn run(
    output: String,
    category: Option<String>,
    project: Option<String>,
    session: Option<String>,
) -> Result<()> {
    let entries = logbook::load_filtered_entries(
        project.as_deref(),
        category.as_deref(),
        session.as_deref(),
    )
    .await?;

    let markdown = render_markdown(&entries, &category, &project, &session);

    if output == "-" {
        print!("{}", markdown);
    } else {
        let path = std::path::Path::new(&output);
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                tokio::fs::create_dir_all(parent).await?;
            }
        }
        tokio::fs::write(&path, &markdown).await?;
    }

    Ok(())
}

/// Render a list of log entries as a Markdown document.
fn render_markdown(
    entries: &[crate::models::LogEntry],
    category: &Option<String>,
    project: &Option<String>,
    session: &Option<String>,
) -> String {
    let mut out = String::new();

    writeln!(out, "# alog Export").unwrap();
    writeln!(out).unwrap();

    // Active filters
    let mut filters: Vec<String> = Vec::new();
    if let Some(p) = project {
        filters.push(format!("project: **{}**", p));
    }
    if let Some(c) = category {
        filters.push(format!("category: **{}**", c));
    }
    if let Some(s) = session {
        filters.push(format!("session: **{}**", s));
    }
    if !filters.is_empty() {
        writeln!(out, "Filters: {}", filters.join(" · ")).unwrap();
        writeln!(out).unwrap();
    }

    if entries.is_empty() {
        writeln!(out, "_No entries matched the given filters._").unwrap();
        return out;
    }

    writeln!(out, "{} entries", entries.len()).unwrap();
    writeln!(out).unwrap();
    writeln!(out, "---").unwrap();
    writeln!(out).unwrap();

    for entry in entries {
        let date = entry.created_at.format("%Y-%m-%d %H:%M UTC");
        let proj = entry.project.as_deref().unwrap_or("global");
        writeln!(out, "## [{cat}] {date}", cat = entry.category, date = date).unwrap();
        writeln!(out).unwrap();
        writeln!(out, "- **ID:** `{}`", entry.id).unwrap();
        writeln!(out, "- **Project:** {}", proj).unwrap();
        if let Some(s) = &entry.session {
            writeln!(out, "- **Session:** {}", s).unwrap();
        }
        writeln!(out).unwrap();
        writeln!(out, "{}", entry.content).unwrap();
        writeln!(out).unwrap();
        writeln!(out, "---").unwrap();
        writeln!(out).unwrap();
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::LogEntry;
    use chrono::Utc;

    fn make_entry(category: &str, content: &str, project: Option<&str>, session: Option<&str>) -> LogEntry {
        LogEntry {
            id: "test-id-1234".into(),
            category: category.into(),
            content: content.into(),
            project: project.map(str::to_string),
            session: session.map(str::to_string),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn test_render_empty_entries() {
        let md = render_markdown(&[], &None, &None, &None);
        assert!(md.contains("No entries matched"));
    }

    #[test]
    fn test_render_includes_content() {
        let entries = vec![make_entry("bugfix", "fixed the thing", None, None)];
        let md = render_markdown(&entries, &None, &None, &None);
        assert!(md.contains("fixed the thing"));
        assert!(md.contains("bugfix"));
        assert!(md.contains("test-id-1234"));
    }

    #[test]
    fn test_render_shows_session_when_present() {
        let entries = vec![make_entry("notes", "session note", None, Some("sess-xyz"))];
        let md = render_markdown(&entries, &None, &None, &Some("sess-xyz".into()));
        assert!(md.contains("sess-xyz"));
    }

    #[test]
    fn test_render_shows_filter_summary() {
        let entries = vec![make_entry("decisions", "chose postgres", Some("myproj"), None)];
        let md = render_markdown(
            &entries,
            &Some("decisions".into()),
            &Some("myproj".into()),
            &None,
        );
        assert!(md.contains("project: **myproj**"));
        assert!(md.contains("category: **decisions**"));
    }

    #[test]
    fn test_render_entry_count() {
        let entries = vec![
            make_entry("notes", "first", None, None),
            make_entry("notes", "second", None, None),
        ];
        let md = render_markdown(&entries, &None, &None, &None);
        assert!(md.contains("2 entries"));
    }

    #[tokio::test]
    async fn test_export_to_stdout_does_not_error() {
        use std::env;
        let tmp = tempfile::tempdir().unwrap();
        env::set_var("HOME", tmp.path());

        // With no entries in the temp HOME, exporting to "-" should succeed
        let result = run("-".into(), None, None, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_export_to_file() {
        use std::env;
        let tmp = tempfile::tempdir().unwrap();
        env::set_var("HOME", tmp.path());

        let out_path = tmp.path().join("export.md");
        let result = run(out_path.to_str().unwrap().to_string(), None, None, None).await;
        assert!(result.is_ok());
        assert!(out_path.exists());
        let content = std::fs::read_to_string(&out_path).unwrap();
        assert!(content.contains("# alog Export"));
    }
}
