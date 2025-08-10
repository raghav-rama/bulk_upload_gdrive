use anyhow::{Ok, Result};
use cli::{Cli, Commands, Parser};
use drive_client::get_drive_client;

use crate::utils::{list_files, upload};

mod cli;
mod drive_client;
mod types;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    let hub = get_drive_client().await?;
    let cli = Cli::parse();
    match &cli.command {
        Commands::List { folder_id } => list_files(hub, folder_id).await?,
        Commands::Upload {
            directory,
            folder_id,
            concurrency,
        } => upload(hub, directory, folder_id, concurrency).await?,
    };
    Ok(())
}
