//! oss sdk
//!
//! 阿里云oss文档：https://help.aliyun.com/zh/oss/
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

pub(crate) mod sign_v4;
pub(crate) mod utils;

pub struct OSSClient {
    access_key_id: String,
    access_key_secret: String,
    endpoint: String,
    region: String,
    bucket: String,
    http_client: reqwest::Client,
}

impl OSSClient {
    pub fn new(
        access_key_id: &str,
        access_key_secret: &str,
        endpoint: &str,
        region: &str,
        bucket: &str,
    ) -> Self {
        OSSClient {
            access_key_id: access_key_id.to_owned(),
            access_key_secret: access_key_secret.to_owned(),
            endpoint: endpoint.to_owned(),
            region: region.to_owned(),
            bucket: bucket.to_owned(),
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
