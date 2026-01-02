//! 阿里云 ECS SDK
//!
//! 云服务器 ECS [文档](https://help.aliyun.com/zh/ecs/developer-reference/api-ecs-2014-05-26-overview)
mod error;
pub use error::Error;

pub mod prefix_list;

use crate::credentials::CredentialsProvider;
use bon::bon;
use std::sync::Arc;
use u_sdk_common::open_api_sign::OpenApiStyle;

pub(crate) const OPENAPI_STYLE: OpenApiStyle = OpenApiStyle::RPC;
pub(crate) const OPENAPI_VERSION: &str = "2014-05-26";

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

pub(crate) async fn parse_json_response<T: serde::de::DeserializeOwned>(
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
