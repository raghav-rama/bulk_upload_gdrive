pub use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "gdrive")]
#[command(author, version, about, long_about=None)]
pub struct Cli {
    /// Directory containing the PDF files
    #[arg(short, long)]
    directory: String,
    /// Folder ID in Google Drive
    #[arg(short, long)]
    folder_id: String,
    /// Max concurrent uploads
    #[arg(short, long, default_value_t = 1000)]
    concurrency: usize,
}
