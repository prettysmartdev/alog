use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "alog", about = "A logbook for AI agents")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Initialize alog for the current git repository
    Init,
    /// Save a new log entry for the given category
    Write {
        /// The category for this log entry
        category: String,
        /// The log entry content
        entry: String,
        /// The project associated with this log entry
        #[arg(long)]
        project: Option<String>,
        /// Add this entry and delete the entry with the given ID
        #[arg(long)]
        replace: Option<String>,
    },
    /// Fuzzy search for log entries
    Recall {
        /// The category to search, or "all" for all categories
        category: String,
        /// The search term
        search_term: String,
        /// Restrict the search to the specific project
        #[arg(long)]
        project: Option<String>,
        /// Maximum number of results to return
        #[arg(long)]
        count: Option<usize>,
        /// Minimum percentage similarity a result must reach (0–100)
        #[arg(long)]
        threshold: Option<u8>,
    },
}
