//! ESA SDK

mod error;
pub use error::Error;

pub(crate) mod utils;

pub mod origin_protection;
pub mod site_management;

use crate::credentials::CredentialsProvider;
use bon::bon;
use std::sync::Arc;
use u_sdk_common::open_api_sign::OpenApiStyle;

pub const OPENAPI_STYLE: OpenApiStyle = OpenApiStyle::RPC;
pub const OPENAPI_VERSION: &str = "2024-09-10";

pub struct Client {
    credentials_provider: Arc<dyn CredentialsProvider>,
    http_client: reqwest::Client,
    host: String,
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
