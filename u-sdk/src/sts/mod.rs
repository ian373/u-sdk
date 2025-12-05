//! STS (Security Token Service) client for assuming roles and obtaining temporary security credentials.
//!
//! [官方文档](https://help.aliyun.com/zh/ram/developer-reference/api-sts-2015-04-01-assumerole)
mod error;
pub mod ram_policy;
mod types;

pub use error::Error;
pub use types::*;

use crate::credentials::CredentialsProvider;
use bon::bon;
use std::sync::Arc;
use u_sdk_common::helper::into_header_map;
use u_sdk_common::open_api_sign::{OpenApiStyle, SignParams, get_openapi_request_header};

//region client
pub struct Client {
    credentials_provider: Arc<dyn CredentialsProvider>,
    http_client: reqwest::Client,
    host: String,
    style: OpenApiStyle,
}

#[bon]
impl Client {
    #[builder(on(String, into))]
    pub fn new(
        credentials_provider: Arc<dyn CredentialsProvider>,
        /// 参数host: [host地址](https://help.aliyun.com/zh/ram/developer-reference/api-sts-2015-04-01-endpoint)
        host: String,
    ) -> Self {
        Self {
            credentials_provider,
            http_client: reqwest::Client::new(),
            host,
            style: OpenApiStyle::RPC,
        }
    }

    pub fn assume_role(&self) -> AssumeRoleBuilder<'_> {
        AssumeRole::builder(self)
    }
}
//endregion

async fn parse_json_response<T: serde::de::DeserializeOwned>(
    resp: reqwest::Response,
) -> Result<T, Error> {
    let status = resp.status();
    if !status.is_success() {
        return Err(Error::RequestAPIFailed {
            status: status.to_string(),
            text: resp.text().await?,
        });
    }

    let bytes = resp.bytes().await?;
    let data: T = serde_json::from_slice(&bytes).map_err(|e| {
        Error::Common(format!(
            "parse response json error: {}, response text: {}",
            e,
            String::from_utf8_lossy(&bytes)
        ))
    })?;

    Ok(data)
}

impl AssumeRole<'_> {
    pub async fn send(&self) -> Result<AssumeRoleResponse, Error> {
        let client = self.client;
        let creds = client.credentials_provider.load().await?;

        let sign_params = SignParams {
            req_method: "GET",
            host: &client.host,
            query_map: self,
            x_acs_action: "AssumeRole",
            x_acs_version: "2015-04-01",
            x_acs_security_token: creds.sts_security_token.as_deref(),
            request_body: None,
            style: &client.style,
        };

        let (common_headers, url_) =
            get_openapi_request_header(&creds.access_key_secret, &creds.access_key_id, sign_params)
                .map_err(|e| Error::Common(format!("get_openapi_request_header error: {}", e)))?;

        let header_map = into_header_map(common_headers);
        let resp = client
            .http_client
            .get(url_)
            .headers(header_map)
            .send()
            .await?;

        let res = parse_json_response(resp).await?;
        Ok(res)
    }
}
