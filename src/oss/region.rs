//! [API 文档](https://help.aliyun.com/zh/oss/developer-reference/describeregions)

use super::utils::into_request_header;
use super::OSSClient;
use crate::error::Error;
use crate::oss::sign_v4::{sign_v4, HTTPVerb};
use crate::utils::common::gmt_format;

use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use url::Url;

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

impl OSSClient {
    /// - `region`: 如果为`None`，则查询所有支持地域对应的Endpoint信息
    pub async fn describe_regions(&self, region: Option<&str>) -> Result<RegionInfoList, Error> {
        let url = Url::parse_with_params(
            &format!("https://{}", self.endpoint),
            [("regions", region.unwrap_or_default())],
        )
        .unwrap();

        let mut canonical_header = BTreeMap::new();
        canonical_header.insert("x-oss-content-sha256", "UNSIGNED-PAYLOAD");
        canonical_header.insert("host", url.host_str().unwrap());

        let mut additional_header = BTreeSet::new();
        additional_header.insert("host");
        let now = time::OffsetDateTime::now_utc();
        let authorization = sign_v4(
            &self.region,
            HTTPVerb::Get,
            &url,
            &canonical_header,
            Some(&additional_header),
            &self.access_key_id,
            &self.access_key_secret,
            &now,
        );

        let mut header = canonical_header.into_iter().collect::<HashMap<_, _>>();
        header.insert("Authorization", &authorization);
        let gmt = gmt_format(now);
        header.insert("Date", &gmt);
        let header_map = into_request_header(header);

        let resp = self.http_client.get(url).headers(header_map).send().await?;
        let text = resp.text().await?;
        let res = quick_xml::de::from_str(&text)?;

        Ok(res)
    }
}
