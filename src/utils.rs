use crate::types::TDriveHub;
use anyhow::Result;
use futures::StreamExt;
use google_drive3::api::File;
use std::{fs::File as FsFile, path::Path, sync::Arc};

pub async fn list_files(hub: TDriveHub, folder_id: &String) -> Result<()> {
    let files = get_files(hub, folder_id).await?;
    if files.is_empty() {
        println!("No files found in that folder (visible to this service account).");
    } else {
        println!("Files in folder {}:", folder_id);
        println!("{}", &files.len());
        for f in files {
            println!(
                "- {} ({})",
                f.name.unwrap_or_default(),
                f.mime_type.unwrap_or_default()
            );
        }
    }
    Ok(())
}

pub async fn get_files(hub: TDriveHub, folder_id: &String) -> Result<Vec<File>> {
    let mut all_files = Vec::new();
    let mut page_token: Option<String> = None;
    loop {
        let mut request = hub
            .files()
            .list()
            .q(&format!("'{}' in parents and trashed=false", &folder_id))
            // optional, if it is a shared drive
            // .supports_all_drives(true)
            // .include_items_from_all_drives(true)
            // .corpora("drive")
            // .drive_id("SHARED_DRIVE_ID")
            // .page_size(100)
            .param("fields", "nextPageToken, files(id, name, mimeType)")
            .add_scope(google_drive3::api::Scope::Full);
        if let Some(token) = page_token {
            request = request.page_token(&token);
        }
        let (_, file_list) = request.doit().await?;
        if let Some(files) = file_list.files {
            all_files.extend(files);
        }
        page_token = file_list.next_page_token;
        if page_token.is_none() {
            break;
        }
    }
    Ok(all_files)
}

pub async fn upload(
    hub: TDriveHub,
    directory: &String,
    folder_id: &String,
    concurrency: &usize,
) -> Result<()> {
    println!("Reading directory: {}", directory);
    println!("Directory exists: {}", Path::new(directory).exists());

    let entries: Vec<_> = std::fs::read_dir(directory)?
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("pdf"))
        .collect();

    entries
        .iter()
        .for_each(|e| println!("Found PDF: {:?}", e.file_name()));
    println!("Total PDF files found: {}", entries.len());

    if entries.is_empty() {
        println!("No PDF files found in directory");
        return Ok(());
    }
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
        .buffer_unordered(*concurrency)
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

    Ok(())
}
