use super::utils::now_gmt;
use super::OSSClient;
use crate::error::Error;
use crate::oss::utils::sign_authorization;

use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    StatusCode,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use url::Url;

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct ListBucketsQueryParams<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    // TODO 这里可以改为Option<u16>，,1~1000，因为max_keys输入的时候是必是数字，
    // 这样写更不容易出错，但这要就需要解决类型问题，因为这个结构体需要序列化，类型必须同一，想办法解决
    pub max_keys: Option<&'a str>,
}

// region:    --- ListBucketResult
/// 如果属性值为`None`，如：`prefix: None`，表示返回的xml中没有该标签`<Prefix/>`。
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ListAllMyBucketsResult {
    pub prefix: Option<String>,
    pub marker: Option<String>,
    pub max_keys: Option<u32>,
    pub is_truncated: Option<bool>,
    pub next_marker: Option<String>,
    pub owner: Owner,
    pub buckets: Buckets,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Owner {
    #[serde(rename = "ID")]
    pub id: String,
    pub display_name: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Buckets {
    pub bucket: Vec<Bucket>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Bucket {
    pub name: String,
    pub comment: String,
    pub creation_date: String,
    pub location: String,
    pub extranet_endpoint: String,
    pub intranet_endpoint: String,
    pub region: String,
    pub storage_class: String,
    // 文档中说有这个字段，实际请求又发现没有...
    // pub resource_group_id: String,
}
// endregion: --- ListBucketResult

impl OSSClient {
    pub async fn list_buckets(
        &self,
        x_oss_resource_group_id: Option<&str>,
        query_params: ListBucketsQueryParams<'_>,
    ) -> Result<ListAllMyBucketsResult, Error> {
        let query_map: HashMap<String, String> =
            serde_json::from_value(serde_json::to_value(query_params).unwrap()).unwrap();

        // 此api不涉及bucket，url使用https://endpoint
        let url = Url::parse_with_params(&self.endpoint_url(), query_map).unwrap();

        let mut oss_header_map = BTreeMap::new();
        if let Some(s) = x_oss_resource_group_id {
            oss_header_map.insert("x-oss-resource-group-id".to_owned(), s.to_owned());
        }

        let now_gmt = now_gmt();
        let authorization = sign_authorization(
            &self.access_key_id,
            &self.access_key_secret,
            "GET",
            None,
            None,
            &now_gmt,
            Some(&oss_header_map),
            None,
            None,
        );

        let mut header_map = HashMap::new();
        header_map.extend(oss_header_map);

        let mut common_header = self.get_common_header_map(&authorization, None, None, &now_gmt);
        // 注意，这里需要改一下Host的值，默认情况下Host的值为`bucket.endpoint`，但此api不涉及bucket，host的值位应为`endpoint`
        common_header.insert("Host".to_owned(), self.endpoint.clone());

        header_map.extend(common_header);

        // 把HashMap转化为reqwest需要的HeaderMap
        let header_map: HeaderMap = header_map
            .iter()
            .map(|(k, v)| {
                let name = HeaderName::from_bytes(k.as_bytes()).unwrap();
                let value = HeaderValue::from_bytes(v.as_bytes()).unwrap();
                (name, value)
            })
            .collect();

        let resp = self.http_client.get(url).headers(header_map).send().await?;
        if resp.status() != StatusCode::OK {
            return Err(Error::StatusCodeNot200Resp(resp));
        }

        let text = resp.text().await?;

        // println!("resp_text:\n{}", resp_text);

        let res = quick_xml::de::from_str(&text)?;

        Ok(res)
    }
}
