mod error;
mod site_management;
mod types_rs;

pub use error::Error;

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

pub async fn parse_json_response<T: serde::de::DeserializeOwned>(
    resp: reqwest::Response,
) -> Result<T, Error> {
    let status = resp.status();

    if !status.is_success() {
        return Err(Error::RequestAPIFailed {
            code: status.to_string(),
            message: resp.text().await?,
        });
    }

    let data = resp.json().await?;
    Ok(data)
}
