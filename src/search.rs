use crate::models::LogEntry;
use strsim::jaro_winkler;

/// Fuzzy search entries by search term.
/// Returns entries sorted by similarity score (descending), filtered to >= threshold (0.0–1.0).
pub fn fuzzy_search<'a>(
    entries: &'a [LogEntry],
    search_term: &str,
    threshold: f64,
) -> Vec<(f64, &'a LogEntry)> {
    let search_lower = search_term.to_lowercase();

    let mut scored: Vec<(f64, &LogEntry)> = entries
        .iter()
        .map(|entry| {
            let score = jaro_winkler(&search_lower, &entry.content.to_lowercase());
            (score, entry)
        })
        .filter(|(score, _)| *score >= threshold)
        .collect();

    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    scored
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_entry(content: &str) -> LogEntry {
        LogEntry {
            id: uuid::Uuid::new_v4().to_string(),
            category: "test".into(),
            content: content.into(),
            project: None,
            session: None,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn test_exact_match_scores_highest() {
        let entries = vec![
            make_entry("hello world"),
            make_entry("foo bar"),
            make_entry("baz qux"),
        ];
        let results = fuzzy_search(&entries, "hello world", 0.0);
        assert!(!results.is_empty());
        assert_eq!(results[0].1.content, "hello world");
        assert!((results[0].0 - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_threshold_filters_low_scores() {
        let entries = vec![
            make_entry("hello world"),
            make_entry("zzzzzzzzzzzzz"),
        ];
        let results = fuzzy_search(&entries, "hello world", 0.9);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1.content, "hello world");
    }

    #[test]
    fn test_results_sorted_descending() {
        let entries = vec![
            make_entry("completely unrelated"),
            make_entry("hello world"),
            make_entry("hello"),
        ];
        let results = fuzzy_search(&entries, "hello world", 0.0);
        assert!(!results.is_empty());
        for i in 1..results.len() {
            assert!(results[i - 1].0 >= results[i].0);
        }
    }

    #[test]
    fn test_empty_entries_returns_empty() {
        let results = fuzzy_search(&[], "hello", 0.0);
        assert!(results.is_empty());
    }

    #[test]
    fn test_threshold_zero_returns_all() {
        let entries = vec![make_entry("alpha"), make_entry("beta"), make_entry("gamma")];
        let results = fuzzy_search(&entries, "alpha", 0.0);
        assert_eq!(results.len(), 3);
    }
}
