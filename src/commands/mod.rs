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
        Commands::Write { category, entry, project, replace } => {
            write::run(category, entry, project, replace).await
        }
        Commands::Recall { category, search_term, project, count, threshold } => {
            recall::run(category, search_term, project, count, threshold).await
        }
    }
}
