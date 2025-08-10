use crate::types::TDriveHub;
use anyhow::{Ok, Result};
use google_drive3::api::FileList;

pub async fn list_files(hub: TDriveHub, folder_id: &String) -> Result<()> {
    let result = get_files(hub, folder_id).await?;

    if let Some(files) = result.files {
        if files.is_empty() {
            println!("No files found in that folder (visible to this service account).");
        } else {
            println!("Files in folder {}:", folder_id);
            for f in files {
                println!(
                    "- {} ({})",
                    f.name.unwrap_or_default(),
                    f.mime_type.unwrap_or_default()
                );
            }
        }
    } else {
        println!("No files array returned.");
    }
    Ok(())
}

pub async fn get_files(hub: TDriveHub, folder_id: &String) -> Result<FileList> {
    let result = hub
        .files()
        .list()
        .q(&format!("'{}' in parents and trashed=false", &folder_id))
        // optional, if it is a shared drive
        // .supports_all_drives(true)
        // .include_items_from_all_drives(true)
        // .corpora("drive")
        // .drive_id("SHARED_DRIVE_ID")
        .page_size(100)
        .param("fields", "nextPageToken, files(id, name, mimeType)")
        .add_scope(google_drive3::api::Scope::Full)
        .doit()
        .await?;
    Ok(result.1)
}
