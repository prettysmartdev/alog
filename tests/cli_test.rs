/// End-to-end tests that invoke the `alog` binary directly.
/// The binary is expected at `target/debug/alog` relative to the workspace root.
use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{LazyLock, Mutex};
use tempfile::TempDir;

/// Serialize tests that modify the HOME environment variable.
static HOME_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

/// Locate the `alog` binary built by cargo.
/// When running via `cargo test`, the test binary lives in `target/debug/deps/`.
/// The `alog` binary is one level up at `target/debug/alog`.
fn alog_binary() -> PathBuf {
    let mut path = env::current_exe().expect("could not find test binary path");
    path.pop(); // remove test binary filename
    if path.ends_with("deps") {
        path.pop(); // target/debug/deps -> target/debug
    }
    path.push("alog");
    path
}

/// Run the alog binary with a given set of arguments and a HOME override.
/// Returns (exit_success, stdout, stderr).
fn run_alog(args: &[&str], home: &TempDir) -> (bool, String, String) {
    let output = Command::new(alog_binary())
        .args(args)
        .env("HOME", home.path())
        .output()
        .expect("failed to execute alog binary");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (output.status.success(), stdout, stderr)
}

#[test]
fn test_help_exits_successfully() {
    let tmp = tempfile::tempdir().unwrap();
    let (ok, stdout, _) = run_alog(&["--help"], &tmp);
    assert!(ok, "alog --help should exit 0");
    assert!(stdout.contains("alog"), "help output should mention the binary name");
}

#[test]
fn test_write_prints_id_and_exits_zero() {
    let _guard = HOME_LOCK.lock().unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let (ok, stdout, _) = run_alog(&["write", "bugfix", "tokio panicked on blocking call"], &tmp);
    assert!(ok, "alog write should exit 0");
    // The ID printed to stdout must be a non-empty UUID-like string
    assert!(!stdout.trim().is_empty(), "alog write should print the new entry ID");
}

#[test]
fn test_recall_after_write_returns_result() {
    let _guard = HOME_LOCK.lock().unwrap();
    let tmp = tempfile::tempdir().unwrap();

    let (ok, _, _) = run_alog(
        &["write", "decisions", "chose postgres for storage backend"],
        &tmp,
    );
    assert!(ok, "write should succeed");

    let (ok, stdout, _) = run_alog(
        &["recall", "decisions", "postgres storage", "--threshold=0"],
        &tmp,
    );
    assert!(ok, "recall should succeed");
    assert!(
        stdout.contains("postgres"),
        "recall should return the stored entry, got: {}",
        stdout
    );
}

#[test]
fn test_recall_all_categories() {
    let _guard = HOME_LOCK.lock().unwrap();
    let tmp = tempfile::tempdir().unwrap();

    run_alog(&["write", "bugfix", "fixed a null pointer"], &tmp);
    run_alog(&["write", "warnings", "watch out for null pointers"], &tmp);

    let (ok, stdout, _) = run_alog(&["recall", "all", "null pointer", "--threshold=0"], &tmp);
    assert!(ok, "recall all should succeed");
    assert!(
        stdout.contains("null pointer"),
        "recall all should find entries across categories, got: {}",
        stdout
    );
}

#[test]
fn test_write_with_project_scopes_entry() {
    let _guard = HOME_LOCK.lock().unwrap();
    let tmp = tempfile::tempdir().unwrap();

    let (ok, _, _) = run_alog(
        &["write", "patterns", "use async/await everywhere", "--project=myrepo"],
        &tmp,
    );
    assert!(ok, "write with --project should succeed");

    // Recall scoped to the same project should find it
    let (ok, stdout, _) = run_alog(
        &["recall", "patterns", "async", "--project=myrepo", "--threshold=0"],
        &tmp,
    );
    assert!(ok);
    assert!(stdout.contains("async"), "project recall should find scoped entry, got: {}", stdout);

    // Recall scoped to a different project should not find it
    let (ok, stdout, _) = run_alog(
        &["recall", "patterns", "async", "--project=otherrepo", "--threshold=0"],
        &tmp,
    );
    assert!(ok);
    assert!(
        !stdout.contains("async/await"),
        "entry from myrepo should not appear under otherrepo, got: {}",
        stdout
    );
}

#[test]
fn test_write_with_session_flag() {
    let _guard = HOME_LOCK.lock().unwrap();
    let tmp = tempfile::tempdir().unwrap();

    let (ok, _, _) = run_alog(
        &["write", "notes", "session tagged entry", "--session=test-session-001"],
        &tmp,
    );
    assert!(ok, "write with --session should succeed");
}

#[test]
fn test_write_session_too_long_fails() {
    let _guard = HOME_LOCK.lock().unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let long_session = "x".repeat(101);

    let (ok, _, stderr) = run_alog(
        &["write", "notes", "content", &format!("--session={}", long_session)],
        &tmp,
    );
    assert!(!ok, "write with overlong session should fail");
    assert!(stderr.contains("100"), "error message should mention the limit, got: {}", stderr);
}

#[test]
fn test_write_replace_removes_old_entry() {
    let _guard = HOME_LOCK.lock().unwrap();
    let tmp = tempfile::tempdir().unwrap();

    // Write an entry and capture its ID
    let (ok, old_id, _) = run_alog(&["write", "decisions", "old decision"], &tmp);
    assert!(ok);
    let old_id = old_id.trim().to_string();
    assert!(!old_id.is_empty(), "expected an ID, got empty string");

    // Replace it
    let replace_flag = format!("--replace={}", old_id);
    let (ok, _, _) = run_alog(&["write", "decisions", "new decision", &replace_flag], &tmp);
    assert!(ok, "write --replace should succeed");

    // Old content should be gone
    let (ok, stdout, _) = run_alog(&["recall", "decisions", "decision", "--threshold=0"], &tmp);
    assert!(ok);
    assert!(stdout.contains("new decision"), "new entry should be present, got: {}", stdout);
    assert!(!stdout.contains("old decision"), "old entry should be gone, got: {}", stdout);
}

#[test]
fn test_export_to_stdout() {
    let _guard = HOME_LOCK.lock().unwrap();
    let tmp = tempfile::tempdir().unwrap();

    run_alog(&["write", "bugfix", "exported bug entry", "--session=exp-sess"], &tmp);

    let (ok, stdout, _) = run_alog(&["export", "-", "--session=exp-sess"], &tmp);
    assert!(ok, "export to stdout should succeed");
    assert!(stdout.contains("# alog Export"), "output should contain Markdown header, got: {}", stdout);
    assert!(stdout.contains("exported bug entry"), "output should contain entry content, got: {}", stdout);
}

#[test]
fn test_export_to_file() {
    let _guard = HOME_LOCK.lock().unwrap();
    let tmp = tempfile::tempdir().unwrap();

    run_alog(&["write", "notes", "file export test entry"], &tmp);

    let out_path = tmp.path().join("out.md");
    let (ok, _, _) = run_alog(
        &["export", out_path.to_str().unwrap()],
        &tmp,
    );
    assert!(ok, "export to file should succeed");
    assert!(out_path.exists(), "output file should be created");
    let content = std::fs::read_to_string(&out_path).unwrap();
    assert!(content.contains("# alog Export"), "file should contain Markdown header");
    assert!(content.contains("file export test entry"), "file should contain entry content");
}

#[test]
fn test_count_limits_recall_results() {
    let _guard = HOME_LOCK.lock().unwrap();
    let tmp = tempfile::tempdir().unwrap();

    for i in 0..5 {
        run_alog(&["write", "notes", &format!("note number {}", i)], &tmp);
    }

    let (ok, stdout, _) = run_alog(&["recall", "notes", "note", "--count=2", "--threshold=0"], &tmp);
    assert!(ok);
    // Each result occupies one line
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();
    assert_eq!(lines.len(), 2, "expected 2 results with --count=2, got: {}", stdout);
}
