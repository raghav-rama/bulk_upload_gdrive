use google_drive3::{DriveHub, hyper_rustls, hyper_util};
use hyper_rustls::HttpsConnector;
use hyper_util::client::legacy::connect::HttpConnector;
use std::sync::Arc;

pub type TDriveHub = Arc<DriveHub<HttpsConnector<HttpConnector>>>;
