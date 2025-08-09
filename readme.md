# Bulk Upload GDrive

A high-performance CLI tool for bulk uploading files to Google Drive. This tool leverages Google Drive API v3 and concurrent uploads to dramatically speed up the upload process compared to the web interface.

## 🚀 Performance

- **Uploaded 1,881 PDFs** (approximately 12 GB) in just **16 minutes**
- Uses concurrent uploads with configurable concurrency (default: 1000 simultaneous uploads)
- Leverages the free `google.apps.drive.v3.DriveFiles.Create` API with generous rate limits (12,000 requests per minute)

## ✨ Features

- **Bulk Upload**: Upload thousands of files efficiently
- **Concurrent Processing**: Configurable concurrent upload streams for maximum speed
- **Service Account Authentication**: Secure authentication using Google service account credentials
- **PDF Support**: Currently optimized for PDF file uploads
- **Progress Tracking**: Real-time upload status for each file
- **Error Handling**: Graceful error handling with detailed error messages

## 📋 Prerequisites

- Google Cloud Project with Drive API enabled
- Service Account with appropriate permissions
- `service_account.json` credentials file

## 🔧 Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/raghav-rama/bulk_upload_gdrive.git
cd bulk_upload_gdrive

# Build the project
cargo build --release

# The binary will be available at ./target/release/bulk_upload_gdrive
```

## 🔑 Setup

1. **Create a Google Cloud Project**:

   - Go to [Google Cloud Console](https://console.cloud.google.com)
   - Create a new project or select existing one
   - Enable Google Drive API

2. **Create Service Account**:

   - Navigate to "IAM & Admin" → "Service Accounts"
   - Create a new service account
   - Download the JSON key file
   - Rename it to `service_account.json` and place it in the project root

3. **Share Target Folder**:
   - In Google Drive, share the target folder with the service account email
   - Grant "Editor" permissions

## 📖 Usage

### Command Line Interface

```bash
gdrive --directory <PATH> --folder-id <FOLDER_ID> [OPTIONS]
```

### Parameters

| Parameter   | Short | Long            | Description                                         | Default  |
| ----------- | ----- | --------------- | --------------------------------------------------- | -------- |
| Directory   | `-d`  | `--directory`   | Path to directory containing PDF files              | Required |
| Folder ID   | `-f`  | `--folder-id`   | Google Drive folder ID where files will be uploaded | Required |
| Concurrency | `-c`  | `--concurrency` | Maximum number of concurrent uploads                | 1000     |

### Examples

```bash
# Basic usage
gdrive -d /path/to/pdfs -f 1eAdXLJZZftHewGRn0H6fLGybj5xgF1Yw

# With custom concurrency limit
gdrive -d /path/to/pdfs -f 1eAdXLJZZftHewGRn0H6fLGybj5xgF1Yw -c 500

# Using long form parameters
gdrive --directory /path/to/pdfs --folder-id 1eAdXLJZZftHewGRn0H6fLGybj5xgF1Yw --concurrency 100
```

### Finding Folder ID

To get the folder ID from Google Drive:

1. Navigate to the folder in Google Drive
2. Look at the URL: `https://drive.google.com/drive/folders/FOLDER_ID`
3. Copy the `FOLDER_ID` portion

## 🛠️ Dependencies

- `google-drive3` - Google Drive API client
- `tokio` - Async runtime
- `futures` - Async stream processing
- `clap` - CLI argument parsing
- `serde` & `serde_json` - JSON serialization
- `mime` - MIME type handling
- `anyhow` - Error handling

## 🏗️ Architecture

The tool uses:

- **Async/Await**: Tokio runtime for asynchronous operations
- **Concurrent Streams**: `futures::stream` with `buffer_unordered` for parallel uploads
- **Service Account Auth**: Secure authentication without user interaction
- **Efficient Memory Usage**: Streams files directly without loading into memory

## 📊 API Limits & Costs

- **API Used**: `google.apps.drive.v3.DriveFiles.Create`
- **Cost**: FREE
- **Rate Limit**: 12,000 requests per minute
- **Daily Limit**: Subject to your Google Cloud project quotas

## 🚧 Roadmap

- [ ] Support for multiple file types (not just PDFs)
- [ ] Resume interrupted uploads
- [ ] Progress bar with ETA
- [ ] Recursive directory upload
- [ ] File filtering options (by size, date, pattern)
- [ ] Dry run mode
- [ ] Upload verification
- [ ] Configurable retry logic
- [ ] Upload to shared drives

## ⚠️ Limitations

- Currently supports only PDF files
- Requires service account authentication
- No GUI interface (CLI only)

## 🙏 Acknowledgments

Powered by Google Drive API v3
