use super::utils::parse_json_response;
use super::{Client, Error, GetIpProtectionResult};
use bon::Builder;
use std::collections::HashMap;
use u_sdk_common::helper::into_header_map;
use u_sdk_common::open_api_sign::{SignParams, get_openapi_request_header};

#[derive(Builder)]
pub struct GetIpProtection<'a> {
    #[builder(start_fn)]
    client: &'a Client,
}

impl GetIpProtection<'_> {
    pub async fn send(&self) -> Result<GetIpProtectionResult, Error> {
        let client = self.client;
        let creds = client.credentials_provider.load().await?;

        let sign_params = SignParams {
            req_method: "GET",
            host: &client.host,
            query_map: HashMap::<&str, &str>::new(),
            x_acs_action: "GetIpProtection",
            x_acs_version: "2015-11-23",
            x_acs_security_token: creds.sts_security_token.as_deref(),
            request_body: None,
            style: &client.style,
        };

        let (headers, url_) =
            get_openapi_request_header(&creds.access_key_secret, &creds.access_key_id, sign_params)
                .map_err(|e| Error::Common(format!("get_common_headers error: {}", e)))?;
        let resp = self
            .client
            .http_client
            .get(url_)
            .headers(into_header_map(headers))
            .send()
            .await?;

        let resp = parse_json_response(resp).await?;
        Ok(resp)
    }
}
