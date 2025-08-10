pub use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "gdrive")]
#[command(author, version, about, long_about=None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Upload {
        /// Directory containing the PDF files
        #[arg(short, long)]
        directory: String,
        /// Folder ID in Google Drive
        #[arg(short = 'f', long)]
        folder_id: String,
        /// Max concurrent uploads
        #[arg(short = 'c', long, default_value_t = 1000)]
        concurrency: usize,
    },
    List {
        #[arg(short = 'f', long)]
        folder_id: String,
    },
}
