use std::fmt;

#[derive(Debug)]
pub enum AlogError {
    Storage(String),
    EntryNotFound(String),
}

impl fmt::Display for AlogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AlogError::Storage(msg) => write!(f, "Storage error: {}", msg),
            AlogError::EntryNotFound(id) => write!(f, "Log entry not found: {}", id),
        }
    }
}

impl std::error::Error for AlogError {}
