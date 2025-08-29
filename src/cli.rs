pub use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(name = "gdrive")]
#[command(author, version, about, long_about=None)]
pub struct Cli {
    #[arg(short = 'a', long, value_enum, default_value_t = AuthMethod::ServiceAccount)]
    pub auth_method: AuthMethod,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum AuthMethod {
    ServiceAccount,
    OAuth,
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
    Download {
        #[arg(short = 'p', long)]
        path: String,
        #[arg(short = 'f', long)]
        folder_id: String,
        /// Max concurrent downloads
        #[arg(short = 'c', long, default_value_t = 50)]
        concurrency: usize,
    },
}
