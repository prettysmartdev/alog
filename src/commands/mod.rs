pub mod export;
pub mod init;
pub mod recall;
pub mod write;

use anyhow::Result;
use crate::cli::{Cli, Commands};

pub async fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Init => {
            init::run().await
        }
        Commands::Write { category, entry, project, replace, session } => {
            write::run(category, entry, project, replace, session).await
        }
        Commands::Recall { category, search_term, project, count, threshold } => {
            recall::run(category, search_term, project, count, threshold).await
        }
        Commands::Export { output, category, project, session } => {
            export::run(output, category, project, session).await
        }
    }
}
