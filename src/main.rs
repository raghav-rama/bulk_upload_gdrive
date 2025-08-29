use anyhow::{Ok, Result};
use cli::{Cli, Commands, Parser};
use drive_client::get_drive_client;

use crate::utils::{download_files, list_files, upload};

mod cli;
mod drive_client;
mod types;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let hub = get_drive_client(&cli.auth_method).await?;
    match &cli.command {
        Commands::List { folder_id } => list_files(hub, folder_id).await?,
        Commands::Upload {
            directory,
            folder_id,
            concurrency,
        } => upload(hub, directory, folder_id, concurrency).await?,
        Commands::Download {
            path,
            folder_id,
            concurrency,
        } => download_files(hub, folder_id, path, *concurrency).await?,
    };
    Ok(())
}
