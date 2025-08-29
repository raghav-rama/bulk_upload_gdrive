use crate::types::TDriveHub;
use anyhow::Result;
use futures::StreamExt;
use google_drive3::api::File;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::{
    collections::HashSet,
    fs::File as FsFile,
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::time::sleep;

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
            // .corpora("allDrives")
            .include_items_from_all_drives(true)
            .supports_all_drives(true)
            .page_size(1000)
            .param("fields", "nextPageToken, files(id, name, mimeType, size)")
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
    let start_time = Instant::now();
    println!("dtarting optimized bulk upload");
    println!("Source directory: {}", directory);
    println!("Target folder ID: {}", folder_id);

    let existing_files = get_existing_files(&hub, folder_id).await?;

    let entries = collect_files_to_upload(directory, &existing_files)?;

    if entries.is_empty() {
        println!("No new files found to upload");
        return Ok(());
    }

    println!("Found {} files to upload", entries.len());

    let total_size: u64 = entries
        .iter()
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum();

    println!(
        "Total size: {:.2} GB",
        total_size as f64 / (1024.0 * 1024.0 * 1024.0)
    );

    let optimal_concurrency = calculate_optimal_concurrency(&entries, *concurrency);
    println!("Using {} concurrent uploads", optimal_concurrency);

    let multi_progress = Arc::new(MultiProgress::new());
    let overall_progress = multi_progress.add(ProgressBar::new(entries.len() as u64));
    overall_progress.set_style(
        ProgressStyle::default_bar()
            .template("Uploading [{bar:40.cyan/blue}] {pos}/{len} files ({percent}%) | {elapsed_precise} | ETA: {eta_precise}")
            .unwrap()
            .progress_chars("█▓▒░ ")
    );

    let success_count = Arc::new(AtomicUsize::new(0));
    let failure_count = Arc::new(AtomicUsize::new(0));
    let retry_count = Arc::new(AtomicUsize::new(0));

    futures::stream::iter(entries)
        .map(|entry| {
            let folder_id = folder_id.to_string();
            let hub = Arc::clone(&hub);
            let progress: Arc<MultiProgress> = Arc::clone(&multi_progress);
            let overall_prog = overall_progress.clone();
            let success = Arc::clone(&success_count);
            let failure = Arc::clone(&failure_count);
            let retries = Arc::clone(&retry_count);

            async move {
                let path = entry.path();
                let fname = path.file_name().unwrap().to_string_lossy().to_string();

                let file_progress = progress.add(ProgressBar::new_spinner());
                file_progress.set_style(
                    ProgressStyle::default_spinner()
                        .template(&format!("  {{spinner}} Uploading: {}...", fname))
                        .unwrap(),
                );
                file_progress.enable_steady_tick(Duration::from_millis(100));

                let result = upload_file_with_retry(
                    &hub, &path, &fname, &folder_id, 3, // max retries
                    &retries,
                )
                .await;

                file_progress.finish_and_clear();
                overall_prog.inc(1);

                match result {
                    Ok(file_id) => {
                        success.fetch_add(1, Ordering::Relaxed);
                        overall_prog.set_message(fname.to_string());
                        (fname, Ok(file_id))
                    }
                    Err(e) => {
                        failure.fetch_add(1, Ordering::Relaxed);
                        overall_prog.set_message(fname.to_string());
                        (fname, Err(e))
                    }
                }
            }
        })
        .buffer_unordered(optimal_concurrency)
        .for_each(|(fname, result)| async move {
            match result {
                Ok(file_id) => {
                    println!("Uploaded '{}' -> ID: {}", fname, file_id);
                }
                Err(e) => {
                    eprintln!("Failed '{}': {}", fname, e);
                }
            }
        })
        .await;

    overall_progress.finish_with_message("Upload complete!");

    let elapsed = start_time.elapsed();
    let success = success_count.load(Ordering::Relaxed);
    let failed = failure_count.load(Ordering::Relaxed);
    let retries = retry_count.load(Ordering::Relaxed);

    println!("\nUpload Summary:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Successful uploads: {}", success);
    println!("Failed uploads: {}", failed);
    println!("Total retries: {}", retries);
    println!("Total time: {:?}", elapsed);
    println!(
        "Average speed: {:.2} files/sec",
        success as f64 / elapsed.as_secs_f64()
    );

    if total_size > 0 {
        let mb_per_sec = (total_size as f64 / (1024.0 * 1024.0)) / elapsed.as_secs_f64();
        println!("Upload speed: {:.2} MB/s", mb_per_sec);
    }

    Ok(())
}

async fn get_existing_files(hub: &TDriveHub, folder_id: &str) -> Result<HashSet<String>> {
    println!("Checking for existing files...");
    let files = get_files(Arc::clone(hub), &folder_id.to_string()).await?;
    let existing: HashSet<String> = files.into_iter().filter_map(|f| f.name).collect();

    if !existing.is_empty() {
        println!(
            "Found {} existing files, will skip duplicates",
            existing.len()
        );
    }

    Ok(existing)
}

fn collect_files_to_upload(
    directory: &str,
    existing_files: &HashSet<String>,
) -> Result<Vec<std::fs::DirEntry>> {
    let supported_extensions = vec![
        "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "txt", "jpg", "jpeg", "png", "gif",
        "mp4", "mp3", "zip",
    ];

    let entries: Vec<_> = std::fs::read_dir(directory)?
        .filter_map(Result::ok)
        .filter(|e| {
            if let Some(ext) = e.path().extension()
                && let Some(ext_str) = ext.to_str()
            {
                let is_supported = supported_extensions.contains(&ext_str.to_lowercase().as_str());
                let filename = e.file_name().to_string_lossy().to_string();
                let is_new = !existing_files.contains(&filename);
                return is_supported && is_new;
            }
            false
        })
        .collect();

    let mut file_types: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for entry in &entries {
        if let Some(ext) = entry.path().extension() {
            *file_types
                .entry(ext.to_string_lossy().to_string())
                .or_insert(0) += 1;
        }
    }

    if !file_types.is_empty() {
        println!("File types found:");
        for (ext, count) in file_types {
            println!("   - .{}: {} files", ext, count);
        }
    }

    Ok(entries)
}

fn calculate_optimal_concurrency(entries: &[std::fs::DirEntry], max_concurrency: usize) -> usize {
    let avg_size: u64 = entries
        .iter()
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum::<u64>()
        .checked_div(entries.len() as u64)
        .unwrap_or(0);

    let optimal = if avg_size < 1_000_000 {
        // < 1MB
        max_concurrency.min(100)
    } else if avg_size < 10_000_000 {
        // < 10MB
        max_concurrency.min(50)
    } else if avg_size < 100_000_000 {
        // < 100MB
        max_concurrency.min(20)
    } else {
        max_concurrency.min(10)
    };

    optimal.max(1)
}

async fn upload_file_with_retry(
    hub: &TDriveHub,
    path: &Path,
    fname: &str,
    folder_id: &str,
    max_retries: u32,
    retry_counter: &Arc<AtomicUsize>,
) -> Result<String> {
    let mut attempt = 0;
    let mut backoff = Duration::from_secs(1);

    loop {
        match upload_single_file(hub, path, fname, folder_id).await {
            Ok(file_id) => return Ok(file_id),
            Err(e) if attempt < max_retries => {
                attempt += 1;
                retry_counter.fetch_add(1, Ordering::Relaxed);

                let is_rate_limit = e.to_string().contains("rateLimitExceeded")
                    || e.to_string().contains("userRateLimitExceeded")
                    || e.to_string().contains("429");

                if is_rate_limit {
                    backoff = Duration::from_secs(10 * attempt as u64);
                    eprintln!(
                        "Rate limit hit for '{}', waiting {:?} before retry {}/{}",
                        fname, backoff, attempt, max_retries
                    );
                } else {
                    eprintln!(
                        "Upload failed for '{}': {}, retrying {}/{} in {:?}",
                        fname, e, attempt, max_retries, backoff
                    );
                }

                sleep(backoff).await;
                backoff *= 2; // Exponential backoff
                backoff = backoff.min(Duration::from_secs(60));
            }
            Err(e) => return Err(e),
        }
    }
}

async fn upload_single_file(
    hub: &TDriveHub,
    path: &Path,
    fname: &str,
    folder_id: &str,
) -> Result<String> {
    let fs_file = FsFile::open(path)?;

    let mime_type = get_mime_type(path);
    let mime: mime::Mime = mime_type
        .parse()
        .unwrap_or_else(|_| "application/octet-stream".parse().unwrap());

    let drive_file = File {
        name: Some(fname.to_string()),
        parents: Some(vec![folder_id.to_string()]),
        ..Default::default()
    };

    let (_, uploaded) = hub
        .files()
        .create(drive_file)
        .add_scope(google_drive3::api::Scope::Full)
        .upload(fs_file, mime)
        .await?;

    Ok(uploaded.id.unwrap_or_default())
}

fn get_mime_type(path: &Path) -> &'static str {
    match path.extension().and_then(|s| s.to_str()) {
        Some("pdf") => "application/pdf",
        Some("doc") => "application/msword",
        Some("docx") => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        Some("xls") => "application/vnd.ms-excel",
        Some("xlsx") => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        Some("ppt") => "application/vnd.ms-powerpoint",
        Some("pptx") => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        Some("txt") => "text/plain",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("mp4") => "video/mp4",
        Some("mp3") => "audio/mpeg",
        Some("zip") => "application/zip",
        _ => "application/octet-stream",
    }
}
