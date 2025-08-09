use google_drive3::{
    DriveHub, hyper_rustls, hyper_util, yup_oauth2, yup_oauth2::ServiceAccountAuthenticator,
};
use std::{path::Path, sync::Arc};

pub async fn get_drive_client() -> Result<
    Arc<DriveHub<hyper_rustls::HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>>>,
    Box<dyn std::error::Error>,
> {
    let sa_key_path = Path::new("service_account.json");
    println!("Looking for service account file at: {:?}", sa_key_path);
    println!("File exists: {}", sa_key_path.exists());
    println!("Reading service account key...");
    let sa_key = match yup_oauth2::read_service_account_key(sa_key_path).await {
        Ok(key) => {
            println!("Successfully loaded service account key");
            key
        }
        Err(e) => {
            println!("Failed to load service account key: {}", e);
            return Err(e.into());
        }
    };
    let auth = ServiceAccountAuthenticator::builder(sa_key).build().await?;
    println!("Authentication setup complete");
    let https = hyper_rustls::HttpsConnectorBuilder::new()
        .with_native_roots()
        .expect("no native root CA certificates found")
        .https_or_http()
        .enable_http1()
        .build();
    let client = hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
        .build(https);
    let mut hub = DriveHub::new(client, auth);
    hub.user_agent("drive-list/0.1".into()); // nice to have, not mandatory
    let hub = Arc::new(hub);
    return Ok(hub);
}
