//! Credentials and CredentialsProvider definitions.
//!
//! 在构建client的时候需要传入实现了CredentialsProvider trait的类型，因为trait `CredentialsProvider`的load方法是异步的，
//! 所以有些像预签名等api本身是同步的，但是需要用到Credentials的时候，可能需要异步获取Credentials，所以这类操作因此也变为异步的。
//!
//! # Example
//! ```no_run
//! use serde::Deserialize;
//! use u_sdk::credentials::{Credentials, CredentialsError, CredentialsProvider};
//! use u_sdk::oss;
//! use std::sync::Arc;
//!
//! #[derive(Deserialize, Debug)]
//! pub struct OssConfig {
//!     pub access_key_id: String,
//!     pub access_key_secret: String,
//!     pub endpoint: String,
//!     pub bucket_name: String,
//!     pub region: String,
//! }
//!
//! pub struct OssCredsProvider {
//!     creds: Credentials,
//! }
//!
//! impl OssCredsProvider {
//!     pub fn new(access_key_id: String, access_key_secret: String) -> Self {
//!         Self {
//!             creds: Credentials::new(access_key_id, access_key_secret, None, None),
//!         }
//!     }
//! }
//!
//! #[async_trait::async_trait]
//! impl CredentialsProvider for OssCredsProvider {
//!     async fn load(&self) -> Result<Credentials, CredentialsError> {
//!         Ok(self.creds.clone())
//!     }
//! }
//!
//! fn get_oss_client() -> oss::Client {
//!     let file_str = std::fs::read_to_string("tests/oss/config.toml").unwrap();
//!     let conf = toml::from_str::<OssConfig>(&file_str).unwrap();
//!     let creds_provider = Arc::new(OssCredsProvider::new(
//!         conf.access_key_id,
//!         conf.access_key_secret,
//!     ));
//!     oss::Client::builder()
//!         .credentials_provider(creds_provider)
//!         .endpoint(conf.endpoint)
//!         .region(conf.region)
//!         .bucket(conf.bucket_name)
//!         .build()
//! }
//! ```

use time::OffsetDateTime;

#[derive(Clone, Debug)]
pub struct Credentials {
    pub access_key_id: String,
    pub access_key_secret: String,
    pub sts_security_token: Option<String>,
    pub expires_at: Option<OffsetDateTime>,
}

impl Credentials {
    pub fn new(
        access_key_id: impl Into<String>,
        access_key_secret: impl Into<String>,
        security_token: Option<String>,
        expires_at: Option<OffsetDateTime>,
    ) -> Self {
        Self {
            access_key_id: access_key_id.into(),
            access_key_secret: access_key_secret.into(),
            sts_security_token: security_token,
            expires_at,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum CredentialsError {
    #[error("failed to load credentials: {0}")]
    Provider(String),
    // 后面需要可以再细分，比如网络错误、STS 错误等
}

#[async_trait::async_trait]
pub trait CredentialsProvider: Send + Sync {
    async fn load(&self) -> Result<Credentials, CredentialsError>;
}
