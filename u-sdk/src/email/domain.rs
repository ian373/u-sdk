use super::utils::parse_json_response;
use super::{Client, Error, QueryDomainByParamResult};
use bon::Builder;
use serde::Serialize;
use u_sdk_common::helper::into_header_map;
use u_sdk_common::open_api_sign::{SignParams, get_openapi_request_header};

#[serde_with::skip_serializing_none]
#[derive(Builder, Serialize)]
#[builder(on(String, into))]
#[serde(rename_all = "PascalCase")]
pub struct QueryDomainByParam<'a> {
    #[builder(start_fn)]
    #[serde(skip_serializing)]
    client: &'a Client,

    page_no: Option<u32>,
    page_size: Option<u16>,
    key_word: Option<String>,
    status: Option<u8>,
}

impl QueryDomainByParam<'_> {
    pub async fn send(&self) -> Result<QueryDomainByParamResult, Error> {
        let creds = self.client.credentials_provider.load().await?;
        let sign_params = SignParams {
            req_method: "GET",
            host: &self.client.host,
            query_map: self,
            x_acs_action: "QueryDomainByParam",
            x_acs_version: "2015-11-23",
            x_acs_security_token: creds.sts_security_token.as_deref(),
            request_body: None,
            style: &self.client.style,
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
