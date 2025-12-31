use super::utils::{de_option_empty_string_as_none, parse_json_response};
use super::{Client, Error, OPENAPI_STYLE, OPENAPI_VERSION};
use bon::Builder;
use serde::Deserialize;
use std::collections::HashMap;
use u_sdk_common::helper::into_header_map;
use u_sdk_common::open_api_sign::{SignParams, get_openapi_request_header};

// region GetOriginProtection request
/// [GetOriginProtection](https://help.aliyun.com/zh/edge-security-acceleration/esa/api-esa-2024-09-10-getoriginprotection)
#[derive(Builder)]
pub struct GetOriginProtection<'a> {
    #[builder(start_fn)]
    pub(crate) client: &'a Client,
    /// 站点 ID，可通过调用 ListSites 接口获取。
    pub(crate) site_id: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetOriginProtectionResponse {
    pub request_id: String,

    #[serde(rename = "CurrentIPWhitelist")]
    pub current_ip_whitelist: IpWhitelist,
    #[serde(rename = "LatestIPWhitelist")]
    pub latest_ip_whitelist: IpWhitelist,

    #[serde(rename = "DiffIPWhitelist")]
    pub diff_ip_whitelist: DiffIpWhitelist,

    #[serde(rename = "RegionalCurrentIPWhitelist")]
    pub regional_current_ip_whitelist: RegionalIpWhitelist,
    #[serde(rename = "RegionalLatestIPWhitelist")]
    pub regional_latest_ip_whitelist: RegionalIpWhitelist,

    #[serde(rename = "RegionalDiffIPWhitelist")]
    pub regional_diff_ip_whitelist: RegionalDiffIpWhitelist,

    pub need_update: bool,
    #[serde(deserialize_with = "de_option_empty_string_as_none")]
    pub origin_protection: Option<Switch>,
    #[serde(deserialize_with = "de_option_empty_string_as_none")]
    pub origin_converge: Option<Switch>,
    pub site_id: i64,

    #[serde(
        rename = "AutoConfirmIPList",
        deserialize_with = "de_option_empty_string_as_none"
    )]
    pub auto_confirm_ip_list: Option<Switch>,
}

#[derive(Debug, Deserialize, Default)]
pub struct IpWhitelist {
    #[serde(rename = "IPv4", default)]
    pub ipv4: Vec<String>,
    #[serde(rename = "IPv6", default)]
    pub ipv6: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct DiffIpWhitelist {
    #[serde(rename = "AddedIPWhitelist", default)]
    // 会出现 `"AddedIPWhitelist": {}` 的情况，所以加上 default
    pub added_ip_whitelist: IpWhitelist,
    // 注意看，这里rename为xxx_Ip_xxx
    #[serde(rename = "NoChangeIpWhitelist", default)]
    pub no_change_ip_whitelist: IpWhitelist,
    #[serde(rename = "RemovedIPWhitelist", default)]
    pub removed_ip_whitelist: IpWhitelist,
}

#[derive(Debug, Deserialize)]
pub struct RegionalIpWhitelist {
    #[serde(rename = "RegionalIPv4", default)]
    pub regional_ipv4: Vec<RegionalCidr>,
    #[serde(rename = "RegionalIPv6", default)]
    pub regional_ipv6: Vec<RegionalCidr>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RegionalCidr {
    pub cidr: String,
    pub region: String,
}

#[derive(Debug, Deserialize)]
pub struct RegionalDiffIpWhitelist {
    #[serde(rename = "AddedIPRegionWhitelist")]
    pub added_ip_region_whitelist: RegionalIpWhitelist,
    #[serde(rename = "NoChangeIpWhitelist")]
    pub no_change_ip_whitelist: RegionalIpWhitelist,
    #[serde(rename = "RemovedIPRegionWhitelist")]
    pub removed_ip_region_whitelist: RegionalIpWhitelist,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Switch {
    On,
    Off,
}
// endregion

impl Client {
    pub fn get_origin_protection(&self) -> GetOriginProtectionBuilder<'_> {
        GetOriginProtection::builder(self)
    }
}

impl GetOriginProtection<'_> {
    pub async fn send(&self) -> Result<GetOriginProtectionResponse, Error> {
        let client = self.client;
        let creds = client.credentials_provider.load().await?;

        let sign_params = SignParams {
            req_method: "GET",
            host: &client.host,
            query_map: HashMap::from([("SiteId", self.site_id.to_string())]),
            x_acs_action: "GetOriginProtection",
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
