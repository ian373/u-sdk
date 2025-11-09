//! [API 文档](https://help.aliyun.com/zh/oss/developer-reference/describeregions)

use super::Client;
use super::utils::{get_request_header_with_bucket_region, parse_xml_response};
use crate::oss::Error;
use crate::oss::sign_v4::HTTPVerb;
use bon::Builder;
use serde::Deserialize;
use std::collections::HashMap;
use url::Url;

#[derive(Builder)]
pub struct DescribeRegions<'a> {
    #[builder(start_fn)]
    pub(crate) client: &'a Client,
    pub(crate) region: Option<&'a str>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct RegionInfoList {
    pub region_info: Option<Vec<RegionInfo>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct RegionInfo {
    pub region: String,
    pub internet_endpoint: String,
    pub internal_endpoint: String,
    pub accelerate_endpoint: String,
}

impl DescribeRegions<'_> {
    /// - `region`: 如果为`None`，则查询所有支持地域对应的Endpoint信息
    pub async fn send(&self) -> Result<RegionInfoList, Error> {
        let client = self.client;

        let request_url = Url::parse_with_params(
            &format!("https://{}", client.endpoint),
            [("regions", self.region.unwrap_or_default())],
        )
        .unwrap();

        let header_map = get_request_header_with_bucket_region(
            client,
            HashMap::with_capacity(0),
            &request_url,
            HTTPVerb::Get,
            &client.region,
            None,
        );

        let resp = client
            .http_client
            .get(request_url)
            .headers(header_map)
            .send()
            .await?;

        let res = parse_xml_response(resp).await?;
        Ok(res)
    }
}

impl Client {
    pub fn describe_regions(&self) -> DescribeRegionsBuilder<'_> {
        DescribeRegions::builder(self)
    }
}
