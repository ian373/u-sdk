//! 阿里云机器翻译sdk

use bon::bon;
use std::sync::Arc;

mod error;
pub use error::Error;

mod trans;
mod types_rs;
mod utils;

use crate::credentials::CredentialsProvider;
pub use types_rs::*;
use u_sdk_common::open_api_sign::OpenApiStyle;

pub struct Client {
    credentials_provider: Arc<dyn CredentialsProvider>,
    http_client: reqwest::Client,
    host: String,
    style: OpenApiStyle,
}

#[bon]
impl Client {
    #[builder(on(String, into))]
    pub fn new(credentials_provider: Arc<dyn CredentialsProvider>, host: String) -> Self {
        Self {
            credentials_provider,
            http_client: reqwest::Client::new(),
            host,
            style: OpenApiStyle::RPC,
        }
    }
}
