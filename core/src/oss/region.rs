//! [API 文档](https://help.aliyun.com/zh/oss/developer-reference/describeregions)

use super::OSSClient;
use super::utils::{handle_response_status, into_request_header};
use crate::error::Error;
use crate::oss::sign_v4::{HTTPVerb, SignV4Param};
use common_lib::helper::gmt_format;

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

/// region API
impl OSSClient {
    /// - `region`: 如果为`None`，则查询所有支持地域对应的Endpoint信息
    pub async fn describe_regions(&self, region: Option<&str>) -> Result<RegionInfoList, Error> {
        let request_url = Url::parse_with_params(
            &format!("https://{}", self.endpoint),
            [("regions", region.unwrap_or_default())],
        )
        .unwrap();

        let mut canonical_header = BTreeMap::new();
        canonical_header.insert("x-oss-content-sha256", "UNSIGNED-PAYLOAD");
        canonical_header.insert("host", request_url.host_str().unwrap());

        let mut additional_header = BTreeSet::new();
        additional_header.insert("host");
        let now = time::OffsetDateTime::now_utc();
        let sign_v4_param = SignV4Param {
            signing_region: &self.region,
            http_verb: HTTPVerb::Get,
            uri: &request_url,
            bucket: None,
            header_map: &canonical_header,
            additional_header: Some(&additional_header),
            date_time: &now,
        };
        let authorization = self.sign_v4(sign_v4_param);

        let mut header = canonical_header.into_iter().collect::<HashMap<_, _>>();
        header.insert("Authorization", &authorization);
        let gmt = gmt_format(&now);
        header.insert("Date", &gmt);
        let header_map = into_request_header(header);

        let resp = self
            .http_client
            .get(request_url)
            .headers(header_map)
            .send()
            .await?;
        let text = handle_response_status(resp).await?;
        let res = quick_xml::de::from_str(&text)?;

        Ok(res)
    }
}
