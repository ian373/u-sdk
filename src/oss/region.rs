use super::utils::now_gmt;
use super::OSSClient;
use crate::error::Error;
use crate::oss::utils::{into_header_map, sign_authorization};

use reqwest::StatusCode;
use serde::Deserialize;
use url::Url;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct RegionInfoList {
    pub region_info: Vec<RegionInfo>,
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
            &self.bucket_url(),
            [("regions", region.unwrap_or(""))].into_iter(),
        )
        .unwrap();
        println!("url: {}", url);

        let now_gmt = now_gmt();
        let authorization = sign_authorization(
            &self.access_key_id,
            &self.access_key_secret,
            "GET",
            None,
            None,
            &now_gmt,
            None,
            None,
            None,
        );

        let mut common_header = self.get_common_header_map(&authorization, None, None, &now_gmt);
        common_header.insert("Host".to_owned(), self.endpoint.clone());

        let header_map = into_header_map(common_header);

        let resp = self.http_client.get(url).headers(header_map).send().await?;
        if resp.status() != StatusCode::OK {
            return Err(Error::StatusCodeNot200Resp(resp));
        }

        let text = resp.text().await?;

        let res = quick_xml::de::from_str(&text).map_err(|e| Error::XMLDeError {
            source: e,
            origin_text: text,
        })?;

        Ok(res)
    }
}
