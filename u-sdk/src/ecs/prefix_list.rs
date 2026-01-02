use super::Client;
use super::parse_json_response;
use super::{Error, OPENAPI_STYLE, OPENAPI_VERSION};
use bon::Builder;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use u_sdk_common::helper::into_header_map;
use u_sdk_common::open_api_sign::{SignParams, get_openapi_request_header};

impl Client {
    pub fn describe_prefix_list_attributes(&self) -> DescribePrefixListAttributesBuilder<'_> {
        DescribePrefixListAttributes::builder(self)
    }
}

/// [DescribePrefixListAttributes](https://help.aliyun.com/zh/ecs/developer-reference/api-ecs-2014-05-26-describeprefixlistattributes)
#[derive(Builder, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct DescribePrefixListAttributes<'a> {
    #[serde(skip_serializing)]
    #[builder(start_fn)]
    pub(crate) client: &'a Client,
    /// 地域 ID。您可以调用 DescribeRegions 查看最新的阿里云地域列表。
    region_id: &'a str,
    /// 前缀列表 ID
    prefix_list_id: &'a str,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DescribePrefixListAttributesResponse {
    #[serde(with = "time::serde::iso8601")]
    pub creation_time: OffsetDateTime,
    pub max_entries: i32,
    pub request_id: String,
    pub description: String,
    pub address_family: AddressFamily,
    pub prefix_list_name: String,
    pub prefix_list_id: String,
    pub entries: Option<EntriesWrapper>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct EntriesWrapper {
    pub entry: Vec<PrefixListEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PrefixListEntry {
    pub description: String,
    pub cidr: String,
}

#[derive(Debug, Deserialize)]
pub enum AddressFamily {
    IPv4,
    IPv6,
}

impl DescribePrefixListAttributes<'_> {
    pub async fn send(&self) -> Result<DescribePrefixListAttributesResponse, Error> {
        let client = self.client;
        let creds = client.credentials_provider.load().await?;

        let sign_params = SignParams {
            req_method: "GET",
            host: &client.host,
            query_map: self,
            x_acs_action: "DescribePrefixListAttributes",
            x_acs_version: OPENAPI_VERSION,
            x_acs_security_token: creds.sts_security_token.as_deref(),
            request_body: None,
            style: &OPENAPI_STYLE,
        };

        let (common_headers, url_) =
            get_openapi_request_header(&creds.access_key_secret, &creds.access_key_id, sign_params)
                .map_err(|e| {
                    Error::Common(format!("failed to get openapi request header: {}", e))
                })?;
        let header_map = into_header_map(common_headers);

        let resp = client
            .http_client
            .get(url_)
            .headers(header_map)
            .send()
            .await?;

        let data = parse_json_response(resp).await?;
        Ok(data)
    }
}
