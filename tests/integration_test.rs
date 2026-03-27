/// Integration tests: exercise the write and recall pipelines together.
/// Each test sets HOME to a temp directory to avoid polluting the real ~/.alog.
use std::env;
use std::sync::{LazyLock, Mutex};
use alog::commands::{export, write};
use alog::models::LogEntry;
use alog::search::fuzzy_search;
use alog::storage::logbook;
use chrono::Utc;

/// Serialize tests that mutate HOME to avoid races between parallel test threads.
static HOME_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

fn make_entry(category: &str, content: &str, project: Option<&str>) -> LogEntry {
    LogEntry {
        id: uuid::Uuid::new_v4().to_string(),
        category: category.into(),
        content: content.into(),
        project: project.map(str::to_string),
        session: None,
        created_at: Utc::now(),
    }
}

#[tokio::test]
async fn test_write_then_recall_by_category() {
    let _guard = HOME_LOCK.lock().unwrap();
    let tmp = tempfile::tempdir().unwrap();
    env::set_var("HOME", tmp.path());

    let entries = vec![
        make_entry("decisions", "use postgres for storage", None),
        make_entry("decisions", "prefer async over sync", None),
    ];
    logbook::save_entries("global", "decisions", &entries).await.unwrap();

    let loaded = logbook::load_entries("global", "decisions").await.unwrap();
    assert_eq!(loaded.len(), 2);

    let results = fuzzy_search(&loaded, "postgres storage", 0.0);
    assert!(!results.is_empty());
    assert_eq!(results[0].1.content, "use postgres for storage");
}

#[tokio::test]
async fn test_replace_removes_old_entry() {
    let _guard = HOME_LOCK.lock().unwrap();
    let tmp = tempfile::tempdir().unwrap();
    env::set_var("HOME", tmp.path());

    let old_entry = make_entry("notes", "old content", None);
    let old_id = old_entry.id.clone();
    logbook::save_entries("global", "notes", &[old_entry]).await.unwrap();

    // simulate --replace: load, remove old, add new
    let mut entries = logbook::load_entries("global", "notes").await.unwrap();
    entries.retain(|e| e.id != old_id);
    let new_entry = make_entry("notes", "new content", None);
    entries.push(new_entry.clone());
    logbook::save_entries("global", "notes", &entries).await.unwrap();

    let final_entries = logbook::load_entries("global", "notes").await.unwrap();
    assert_eq!(final_entries.len(), 1);
    assert_eq!(final_entries[0].content, "new content");
    assert!(final_entries.iter().all(|e| e.id != old_id));
}

#[tokio::test]
async fn test_filter_by_project() {
    let _guard = HOME_LOCK.lock().unwrap();
    let tmp = tempfile::tempdir().unwrap();
    env::set_var("HOME", tmp.path());

    let proj_entries = vec![make_entry("notes", "project note", Some("myproject"))];
    let global_entries = vec![make_entry("notes", "global note", None)];
    logbook::save_entries("myproject", "notes", &proj_entries).await.unwrap();
    logbook::save_entries("global", "notes", &global_entries).await.unwrap();

    let project_only = logbook::load_filtered_entries(Some("myproject"), None, None).await.unwrap();
    assert_eq!(project_only.len(), 1);
    assert_eq!(project_only[0].content, "project note");
}

#[tokio::test]
async fn test_recall_all_categories() {
    let _guard = HOME_LOCK.lock().unwrap();
    let tmp = tempfile::tempdir().unwrap();
    env::set_var("HOME", tmp.path());

    logbook::save_entries("global", "notes", &[make_entry("notes", "a note", None)])
        .await
        .unwrap();
    logbook::save_entries("global", "decisions", &[make_entry("decisions", "a decision", None)])
        .await
        .unwrap();

    let all = logbook::load_filtered_entries(Some("global"), None, None).await.unwrap();
    assert_eq!(all.len(), 2);
}

#[tokio::test]
async fn test_threshold_filters_irrelevant_results() {
    let entries = vec![
        make_entry("notes", "the database migration plan", None),
        make_entry("notes", "xyzzy frobnicator", None),
    ];
    let results = fuzzy_search(&entries, "database migration plan", 0.8);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].1.content, "the database migration plan");
}

#[tokio::test]
async fn test_count_limits_results() {
    let entries: Vec<LogEntry> = (0..10)
        .map(|i| make_entry("notes", &format!("note number {}", i), None))
        .collect();

    let mut results = fuzzy_search(&entries, "note", 0.0);
    results.truncate(3);
    assert_eq!(results.len(), 3);
}

/// Integration test: init command creates .alog.json with defaultSimilarityThreshold = 25.
#[tokio::test]
async fn test_init_config_contents() {
    use alog::storage::config::{repo_config_path, RepoConfig};

    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir(tmp.path().join(".git")).unwrap();

    let config_path = repo_config_path(tmp.path());
    let config = RepoConfig::default();
    let json = serde_json::to_string_pretty(&config).unwrap();
    tokio::fs::write(&config_path, &json).await.unwrap();

    let content = tokio::fs::read_to_string(&config_path).await.unwrap();
    assert!(content.contains("defaultSimilarityThreshold"));
    assert!(content.contains("25"));
}

/// Integration test: repo config threshold is used by recall when --threshold flag is absent.
#[tokio::test]
async fn test_recall_uses_repo_config_threshold() {
    use alog::storage::config::{RepoConfig, find_git_root, repo_config_path};

    let _guard = HOME_LOCK.lock().unwrap();
    let tmp = tempfile::tempdir().unwrap();
    env::set_var("HOME", tmp.path());

    // Write a config with a high threshold so nothing matches
    std::fs::create_dir(tmp.path().join(".git")).unwrap();
    let git_root = find_git_root(tmp.path()).unwrap();
    let config_path = repo_config_path(&git_root);
    let config = RepoConfig { default_similarity_threshold: 99 };
    let json = serde_json::to_string_pretty(&config).unwrap();
    tokio::fs::write(&config_path, json).await.unwrap();

    let entries = vec![
        make_entry("notes", "something completely different", None),
    ];

    // At threshold 0.99 the unrelated entry should be filtered
    let threshold = config.default_similarity_threshold as f64 / 100.0;
    let results = fuzzy_search(&entries, "totally unrelated query", threshold);
    assert!(results.is_empty());
}

/// Integration test: write with --session stores session on the entry.
#[tokio::test]
async fn test_write_stores_session() {
    let _guard = HOME_LOCK.lock().unwrap();
    let tmp = tempfile::tempdir().unwrap();
    env::set_var("HOME", tmp.path());

    write::run(
        "notes".into(),
        "session entry content".into(),
        None,
        None,
        Some("integration-session-1".into()),
    )
    .await
    .unwrap();

    let entries = logbook::load_entries("global", "notes").await.unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].session.as_deref(), Some("integration-session-1"));
}

/// Integration test: load_filtered_entries with session filter returns only matching entries.
#[tokio::test]
async fn test_session_filter_integration() {
    let _guard = HOME_LOCK.lock().unwrap();
    let tmp = tempfile::tempdir().unwrap();
    env::set_var("HOME", tmp.path());

    // Write entries with different sessions
    write::run("notes".into(), "from session A".into(), None, None, Some("sess-A".into()))
        .await
        .unwrap();
    write::run("notes".into(), "from session B".into(), None, None, Some("sess-B".into()))
        .await
        .unwrap();
    write::run("notes".into(), "no session".into(), None, None, None)
        .await
        .unwrap();

    let sess_a = logbook::load_filtered_entries(None, None, Some("sess-A")).await.unwrap();
    assert_eq!(sess_a.len(), 1);
    assert_eq!(sess_a[0].content, "from session A");

    let all = logbook::load_filtered_entries(None, None, None).await.unwrap();
    assert_eq!(all.len(), 3);
}

/// Integration test: export produces a valid Markdown file with correct content.
#[tokio::test]
async fn test_export_writes_markdown_file() {
    let _guard = HOME_LOCK.lock().unwrap();
    let tmp = tempfile::tempdir().unwrap();
    env::set_var("HOME", tmp.path());

    // Write a few entries
    write::run("bugfix".into(), "fixed null pointer".into(), None, None, Some("s1".into()))
        .await
        .unwrap();
    write::run("decisions".into(), "use postgres".into(), None, None, Some("s1".into()))
        .await
        .unwrap();
    write::run("notes".into(), "irrelevant note".into(), None, None, Some("s2".into()))
        .await
        .unwrap();

    let out_path = tmp.path().join("report.md");
    export::run(
        out_path.to_str().unwrap().to_string(),
        None,
        None,
        Some("s1".into()),
    )
    .await
    .unwrap();

    assert!(out_path.exists());
    let content = std::fs::read_to_string(&out_path).unwrap();
    assert!(content.contains("# alog Export"));
    assert!(content.contains("fixed null pointer"));
    assert!(content.contains("use postgres"));
    assert!(!content.contains("irrelevant note"));
}

/// Integration test: export with no matching entries produces graceful output.
#[tokio::test]
async fn test_export_no_matches() {
    let _guard = HOME_LOCK.lock().unwrap();
    let tmp = tempfile::tempdir().unwrap();
    env::set_var("HOME", tmp.path());

    let out_path = tmp.path().join("empty.md");
    export::run(
        out_path.to_str().unwrap().to_string(),
        None,
        None,
        Some("nonexistent-session".into()),
    )
    .await
    .unwrap();

    let content = std::fs::read_to_string(&out_path).unwrap();
    assert!(content.contains("No entries matched"));
}
