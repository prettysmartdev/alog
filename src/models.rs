use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: String,
    pub category: String,
    pub content: String,
    pub project: Option<String>,
    pub session: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl LogEntry {
    pub fn new(
        category: String,
        content: String,
        project: Option<String>,
        session: Option<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            category,
            content,
            project,
            session,
            created_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_entry_has_unique_ids() {
        let a = LogEntry::new("cat".into(), "entry a".into(), None, None);
        let b = LogEntry::new("cat".into(), "entry b".into(), None, None);
        assert_ne!(a.id, b.id);
    }

    #[test]
    fn test_new_entry_stores_fields() {
        let entry = LogEntry::new(
            "notes".into(),
            "hello world".into(),
            Some("myproject".into()),
            Some("sess-001".into()),
        );
        assert_eq!(entry.category, "notes");
        assert_eq!(entry.content, "hello world");
        assert_eq!(entry.project, Some("myproject".into()));
        assert_eq!(entry.session, Some("sess-001".into()));
    }

    #[test]
    fn test_new_entry_session_is_optional() {
        let entry = LogEntry::new("notes".into(), "no session".into(), None, None);
        assert!(entry.session.is_none());
    }
}
