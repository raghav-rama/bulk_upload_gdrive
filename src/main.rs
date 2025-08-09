use futures::StreamExt;
use google_drive3::api::File;
use std::sync::Arc;
use std::{fs::File as FsFile, path::Path};

use drive_client::get_drive_client;

mod drive_client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let hub = get_drive_client().await?;
    let dir = "/Users/ritz/Downloads/Aubrai papers";
    println!("Reading directory: {}", dir);
    println!("Directory exists: {}", Path::new(dir).exists());

    let entries: Vec<_> = std::fs::read_dir(dir)?
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("pdf"))
        .collect();

    entries
        .iter()
        .for_each(|e| println!("📄 Found PDF: {:?}", e.file_name()));
    println!("Total PDF files found: {}", entries.len());

    if entries.is_empty() {
        println!("No PDF files found in directory");
        return Ok(());
    }
    let folder_id = "1eAdXLJZZftHewGRn0H6fLGybj5xgF1Yw";
    println!("Target Google Drive folder ID: {}", folder_id);

    futures::stream::iter(entries)
        .map(|entry| {
            let folder_id = folder_id.to_string();
            let hub = Arc::clone(&hub);
            async move {
                let path = entry.path();
                let fname = path.file_name().unwrap().to_string_lossy().to_string();
                println!("Starting upload for: {}", fname);
                let fs_file = match FsFile::open(&path) {
                    Ok(f) => f,
                    Err(e) => {
                        println!("Failed to open file {}: {}", fname, e);
                        return (fname, Err(e.into()));
                    }
                };
                let mime: mime::Mime = "application/pdf".parse().unwrap();

                let drive_file = File {
                    name: Some(fname.clone()),
                    parents: Some(vec![folder_id.clone()]),
                    ..Default::default()
                };

                let result = hub
                    .files()
                    .create(drive_file)
                    .add_scope(google_drive3::api::Scope::Full)
                    .upload(fs_file, mime.clone())
                    .await;

                println!("Upload attempt completed for: {}", fname);
                (fname, result)
            }
        })
        .buffer_unordered(100)
        .for_each(|(fname, result)| async move {
            match result {
                Ok((_, uploaded)) => {
                    println!(
                        "Uploaded '{}' -> ID {}",
                        fname,
                        uploaded.id.unwrap_or_default()
                    );
                }
                Err(e) => {
                    println!("Failed '{}': {}", fname, e);
                }
            }
        })
        .await;

    // let folder_id = "1mQikqFDq82gaRj8mdUYkMYJWk4wc2RYB";
    // let folder_id = "1p1_tkeh4pRKSTEOz4oKn9apK-McFADLP";
    // let result = hub
    //     .files()
    //     .list()
    //     .q(&format!("'{}' in parents and trashed=false", folder_id))
    // optional, if it is a shared drive
    // .supports_all_drives(true)
    // .include_items_from_all_drives(true)
    // .corpora("drive")
    // .drive_id("SHARED_DRIVE_ID")
    //     .page_size(100)
    //     .param("fields", "nextPageToken, files(id, name, mimeType)")
    //     .add_scope(google_drive3::api::Scope::Full)
    //     .doit()
    //     .await?;

    // if let Some(files) = result.1.files {
    //     if files.is_empty() {
    //         println!("No files found in that folder (visible to this service account).");
    //     } else {
    //         println!("Files in folder {}:", folder_id);
    //         for f in files {
    //             println!(
    //                 "- {} ({})",
    //                 f.name.unwrap_or_default(),
    //                 f.mime_type.unwrap_or_default()
    //             );
    //         }
    //     }
    // } else {
    //     println!("No files array returned.");
    // }
    Ok(())
}
