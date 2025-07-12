//! oss sdk
//!
//! 阿里云oss文档：<https://help.aliyun.com/zh/oss/>
//!
//! 注意：
//!
//! - 所有的api暂时无法使用STS方式（需要添加`x-oss-security-token`）
//! - 所有api目前只能使用Header携带签名的方式请求，暂不支持url参数签名

// 所有api的请求逻辑和操作都基本相同，具体逻辑或步骤可参考service.rs的注释理解

pub mod bucket;
pub mod object;
pub mod region;
// pub mod service;

mod error;
pub use error::Error;

pub(crate) mod sign_v4;
pub(crate) mod utils;

use bon::bon;

pub struct Client {
    access_key_id: String,
    access_key_secret: String,
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
        access_key_id: String,
        access_key_secret: String,
        endpoint: String,
        region: String,
        bucket: String,
    ) -> Self {
        Self {
            access_key_id,
            access_key_secret,
            endpoint,
            region,
            bucket,
            http_client: reqwest::Client::new(),
        }
    }

    pub fn set_bucket_info(
        &mut self,
        bucket: Option<&str>,
        region: Option<&str>,
        endpoint: Option<&str>,
    ) {
        if let Some(s) = bucket {
            s.clone_into(&mut self.bucket);
        }
        if let Some(s) = region {
            s.clone_into(&mut self.region);
        }
        if let Some(s) = endpoint {
            s.clone_into(&mut self.endpoint);
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
