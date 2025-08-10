use anyhow::Result;
use google_drive3::{
    DriveHub, hyper_rustls, hyper_util, yup_oauth2,
    yup_oauth2::{
        InstalledFlowAuthenticator, InstalledFlowReturnMethod, ServiceAccountAuthenticator,
        read_application_secret,
    },
};
use std::{path::Path, sync::Arc};

use crate::{cli::AuthMethod, types::TDriveHub};

pub async fn get_drive_client(auth_method: &AuthMethod) -> Result<TDriveHub> {
    match auth_method {
        AuthMethod::OAuth => {
            println!("Using OAuth");
            get_oauth_client().await
        }
        AuthMethod::ServiceAccount => {
            println!("Using service account");
            get_service_account_client().await
        }
    }
}

pub async fn get_service_account_client() -> Result<TDriveHub> {
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
    Ok(hub)
}

pub async fn get_oauth_client() -> Result<TDriveHub> {
    let oauth_credentials = "credentials.json";
    let secret = read_application_secret(oauth_credentials).await?;
    let auth = InstalledFlowAuthenticator::builder(secret, InstalledFlowReturnMethod::HTTPRedirect)
        .persist_tokens_to_disk("token.json")
        .build()
        .await?;
    let client = hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
        .build(
            hyper_rustls::HttpsConnectorBuilder::new()
                .with_native_roots()
                .expect("failed to load platform certs")
                .https_or_http()
                .enable_http1()
                .build(),
        );
    let mut hub = DriveHub::new(client, auth);
    hub.user_agent("drive-list/0.1".into()); // nice to have, not mandatory
    let hub = Arc::new(hub);
    Ok(hub)
}
