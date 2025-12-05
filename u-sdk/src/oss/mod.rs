//! 阿里云oss sdk

pub mod bucket;
pub mod object;
pub mod region;
pub mod service;

mod error;

pub use error::Error;
use std::sync::Arc;

pub(crate) mod sign_v4;
pub(crate) mod utils;

use crate::credentials::CredentialsProvider;
use bon::bon;

pub struct Client {
    credentials_provider: Arc<dyn CredentialsProvider>,
    endpoint: String,
    region: String,
    bucket: String,
    http_client: reqwest::Client,
}

/// 创建oss客户端
#[bon]
impl Client {
    /// region和endpoint：<https://help.aliyun.com/zh/oss/user-guide/regions-and-endpoints>
    #[builder(on(String, into))]
    pub fn new(
        credentials_provider: Arc<dyn CredentialsProvider>,
        endpoint: String,
        region: String,
        bucket: String,
    ) -> Self {
        Self {
            credentials_provider,
            endpoint,
            region,
            bucket,
            http_client: reqwest::Client::new(),
        }
    }
    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    pub fn region(&self) -> &str {
        &self.region
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }
}
