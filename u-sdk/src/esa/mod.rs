//! ESA SDK
//!
//! API 文档地址: <https://api.aliyun.com/product/ESA>

mod error;

pub use error::Error;
use std::fmt::{Debug, Formatter};

pub(crate) mod utils;

pub mod dns_record;
pub mod origin_protection;
pub mod site_management;

use crate::credentials::CredentialsProvider;
use bon::bon;
use std::sync::Arc;
use u_sdk_common::open_api_sign::OpenApiStyle;

pub(crate) const OPENAPI_STYLE: OpenApiStyle = OpenApiStyle::RPC;
pub(crate) const OPENAPI_VERSION: &str = "2024-09-10";

pub struct Client {
    credentials_provider: Arc<dyn CredentialsProvider>,
    http_client: reqwest::Client,
    host: String,
}

impl Debug for Client {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client").field("host", &self.host).finish()
    }
}

#[bon]
impl Client {
    #[builder(on(String, into))]
    pub fn new(credentials_provider: Arc<dyn CredentialsProvider>, host: String) -> Self {
        Self {
            credentials_provider,
            http_client: reqwest::Client::new(),
            host,
        }
    }
}
