//! 阿里云oss sdk
//!
//! 注意：
//!
//! - 所有的api暂时无法使用STS方式（需要添加`x-oss-security-token`）
//! - 所有api目前只能使用Header携带签名的方式请求，暂不支持url参数签名

// 所有api的请求逻辑和操作都基本相同，具体逻辑或步骤可参考service.rs的注释理解

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
