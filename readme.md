# Bulk Upload GDrive

A high-performance CLI tool for bulk uploading files to Google Drive. This tool leverages Google Drive API v3 and concurrent uploads to dramatically speed up the upload process compared to the web interface.

## 🚀 Performance

- **Uploaded 1,881 PDFs** (approximately 12 GB) in just **16 minutes**
- Uses concurrent uploads with configurable concurrency (default: 1000 simultaneous uploads)
- Leverages the free `google.apps.drive.v3.DriveFiles.Create` API with generous rate limits (12,000 requests per minute)

## ✨ Features

- **Bulk Upload**: Upload thousands of files efficiently
- **Dual Authentication**: Support for both OAuth 2.0 and Service Account authentication
- **Concurrent Processing**: Configurable concurrent upload streams for maximum speed
- **PDF Support**: Currently optimized for PDF file uploads
- **Progress Tracking**: Real-time upload status for each file
- **Error Handling**: Graceful error handling with detailed error messages

## 📋 Prerequisites

- Google Cloud Project with Drive API enabled
- Authentication credentials (either OAuth or Service Account)
- Rust toolchain installed

## 🔑 Authentication Methods

This tool supports two authentication methods, each with different use cases and storage implications:

### OAuth 2.0 Authentication

**Best for:** Personal use, when you want to upload files using your own Google Drive storage quota

**Storage:** Uses YOUR personal Google Drive storage quota

**Setup:** Requires browser-based authentication on first run

### Service Account Authentication

**Best for:** Automation, server environments, or when user interaction isn't possible

**Storage:** Uses the service account's 15GB storage quota (NOT your personal storage)

**Setup:** No browser required, uses a key file

> [!IMPORTANT]
> **Storage Quota Differences:**
>
> - **OAuth**: Files count against YOUR Google Drive storage (unlimited if you have Google One)
> - **Service Account**: Files count against the service account's 15GB quota
>
> If you're getting `storageQuotaExceeded` errors with a service account, switch to OAuth authentication to use your personal storage.

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

## 🔐 Setting Up Authentication

### Option 1: OAuth 2.0 Setup (Uses Your Personal Storage)

1. **Create OAuth Credentials:**
   - Go to [Google Cloud Console](https://console.cloud.google.com/apis/credentials)
   - Create or select your project
   - Enable Google Drive API (APIs & Services → Library → Search "Google Drive API")
2. **Configure OAuth Consent Screen:**
   - Go to APIs & Services → OAuth consent screen
   - Choose "External" for personal use
   - Fill in application name and contact information
   - Add test users if in testing mode
3. **Create OAuth Client ID:**
   - Go to APIs & Services → Credentials
   - Click "Create Credentials" → "OAuth client ID"
   - Application type: **Desktop app**
   - Name it (e.g., "Drive Uploader")
   - Click "Create"
4. **Download Credentials:**
   - Click "Download JSON" on the created credential
   - Save as `credentials.json` in the project root
5. **First Run:**
   - The tool will open your browser for authentication
   - Authorize the application
   - Token will be saved in `token.json` for future use

**credentials.json structure:**

```json
{
  "installed": {
    "client_id": "YOUR_CLIENT_ID.apps.googleusercontent.com",
    "project_id": "your-project-id",
    "auth_uri": "https://accounts.google.com/o/oauth2/auth",
    "token_uri": "https://oauth2.googleapis.com/token",
    "client_secret": "YOUR_CLIENT_SECRET",
    "redirect_uris": ["http://localhost"]
  }
}
```

### Option 2: Service Account Setup (Uses Service Account's 15GB)

1. **Create Service Account:**
   - Go to [Google Cloud Console - Service Accounts](https://console.cloud.google.com/iam-admin/serviceaccounts)
   - Select your project
   - Click "+ Create Service Account"
   - Give it a name and description
   - Click "Create and Continue"
2. **Assign Permissions:**
   - Grant necessary roles (e.g., "Editor" or specific Drive roles)
   - Click "Continue" then "Done"
3. **Create Key:**
   - Click on your new service account
   - Go to "Keys" tab
   - Click "Add Key" → "Create new key"
   - Choose **JSON** format
   - Click "Create"
4. **Download and Configure:**
   - Download the JSON key file
   - Rename to `service_account.json`
   - Place in the project root
5. **Share Drive Folder:**
   - In Google Drive, share the target folder with the service account email
   - Find the email in `service_account.json` (e.g., `name@project.iam.gserviceaccount.com`)
   - Grant "Editor" permissions

**service_account.json structure:**

```json
{
  "type": "service_account",
  "project_id": "your-project-id",
  "private_key_id": "key-id",
  "private_key": "-----BEGIN PRIVATE KEY-----\n...\n-----END PRIVATE KEY-----\n",
  "client_email": "service-account@project.iam.gserviceaccount.com",
  "client_id": "123456789",
  "auth_uri": "https://accounts.google.com/o/oauth2/auth",
  "token_uri": "https://oauth2.googleapis.com/token"
}
```

## 📖 Usage

### Command Line Interface

```bash
gdrive --auth-method <METHOD> <COMMAND> [OPTIONS]
```

### Parameters

| Parameter   | Short | Long            | Description                                         | Default         |
| ----------- | ----- | --------------- | --------------------------------------------------- | --------------- |
| Auth Method | `-a`  | `--auth-method` | Authentication method (o-auth or service-account)   | service-account |
| Directory   | `-d`  | `--directory`   | Path to directory containing PDF files              | Required        |
| Folder ID   | `-f`  | `--folder-id`   | Google Drive folder ID where files will be uploaded | Required        |
| Concurrency | `-c`  | `--concurrency` | Maximum number of concurrent uploads                | 1000            |

### Commands

- `upload` - Upload files to Google Drive
- `list` - List files in a Google Drive folder

### Examples

#### Using OAuth (Your Personal Storage)

```bash
# Upload with OAuth authentication
gdrive --auth-method o-auth upload -d /path/to/pdfs -f FOLDER_ID

# Short form
gdrive -a o-auth upload -d /path/to/pdfs -f FOLDER_ID -c 500

# List files with OAuth
gdrive -a o-auth list -f FOLDER_ID
```

#### Using Service Account (15GB Limit)

```bash
# Upload with service account (default)
gdrive upload -d /path/to/pdfs -f FOLDER_ID

# Explicitly specify service account
gdrive --auth-method service-account upload -d /path/to/pdfs -f FOLDER_ID

# With custom concurrency
gdrive upload -d /path/to/pdfs -f FOLDER_ID -c 100
```

### Finding Folder ID

To get the folder ID from Google Drive:

1. Navigate to the folder in Google Drive
2. Look at the URL: `https://drive.google.com/drive/folders/FOLDER_ID`
3. Copy the `FOLDER_ID` portion

## 🤔 Which Authentication Method Should I Use?

### Use OAuth When:

- You're uploading files for personal use
- You need more than 15GB of storage
- You have a Google One subscription with extra storage
- You want files to appear in your personal Drive
- You're okay with browser-based authentication

### Use Service Account When:

- You need fully automated uploads (no browser interaction)
- You're running on a server or CI/CD pipeline
- You're uploading less than 15GB total
- You don't mind files being owned by the service account
- You need to run scheduled/background tasks

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
- **Dual Authentication**: Support for both OAuth and Service Account flows
- **Efficient Memory Usage**: Streams files directly without loading into memory

## 📊 API Limits & Costs

- **API Used**: `google.apps.drive.v3.DriveFiles.Create`
- **Cost**: FREE
- **Rate Limit**: 12,000 requests per minute
- **Daily Limit**: Subject to your Google Cloud project quotas

## 🚧 Troubleshooting

### "storageQuotaExceeded" Error

**Problem:** Service account has reached its 15GB limit

**Solutions:**

1. Switch to OAuth authentication to use your personal storage
2. Delete files from the service account's Drive
3. Use a different service account
4. Upgrade to Google Workspace for more service account storage

### "Unauthorized" Error

**Problem:** Authentication credentials are invalid or missing

**Solutions:**

1. Ensure `credentials.json` (OAuth) or `service_account.json` (Service Account) exists
2. Check that the files are in the project root directory
3. For service accounts, verify the folder is shared with the service account email
4. For OAuth, try deleting `token.json` and re-authenticating

## 🚧 Roadmap

- [x] OAuth 2.0 authentication support
- [ ] Support for multiple file types (not just PDFs)
- [ ] Resume interrupted uploads
- [ ] Progress bar with ETA
- [ ] Recursive directory upload
- [ ] File filtering options (by size, date, pattern)
- [ ] Dry run mode
- [ ] Upload verification
- [ ] Configurable retry logic
- [ ] Upload to shared drives
- [ ] Domain-wide delegation support

## ⚠️ Limitations

- Currently supports only PDF files
- No GUI interface (CLI only)
- Service accounts limited to 15GB storage
- OAuth requires browser for initial authentication

## 🙏 Acknowledgments

Powered by Google Drive API v3
