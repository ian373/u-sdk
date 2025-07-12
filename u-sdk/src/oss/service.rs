use super::Client;
use super::sign_v4::HTTPVerb;
use super::utils::parse_xml_response;
use crate::oss::Error;
use bon::Builder;
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use std::collections::HashMap;
use url::Url;

#[serde_as]
#[serde_with::skip_serializing_none]
#[derive(Builder, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct ListBuckets<'a> {
    #[builder(start_fn)]
    #[serde(skip_serializing)]
    pub(crate) client: &'a Client,
    // 请求头
    #[serde(skip_serializing)]
    pub(crate) x_oss_resource_group_id: Option<&'a str>,
    // 请求参数
    pub(crate) prefix: Option<&'a str>,
    pub(crate) marker: Option<&'a str>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub(crate) max_keys: Option<u16>,
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
    // 原来的xml中包含了buckets->Option<Vec<Bucket>这种结构，这里使用自定义反序列化函数来减少嵌套
    #[serde(deserialize_with = "unwrap_buckets")]
    pub buckets: Vec<Bucket>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct Buckets {
    #[serde(default)]
    // 这里按照xml结构应该为 `bucket: Option<Vec<Bucket>>`，这里使用default来简化
    bucket: Vec<Bucket>,
}

fn unwrap_buckets<'de, D>(deserializer: D) -> Result<Vec<Bucket>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let helper = Buckets::deserialize(deserializer)?;
    Ok(helper.bucket)
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
pub struct Bucket {
    pub name: String,
    pub creation_date: String,
    pub location: String,
    pub extranet_endpoint: String,
    pub intranet_endpoint: String,
    pub region: String,
    pub storage_class: String,
    pub resource_group_id: Option<String>,
}
// endregion: --- ListBucketResult

impl ListBuckets<'_> {
    pub async fn send(&self) -> Result<ListAllMyBucketsResult, Error> {
        // 构建url的query部分，用于传递url以便签名构建canonical_query_string
        let query_map: HashMap<String, String> =
            serde_json::from_value(serde_json::to_value(self).unwrap()).unwrap();

        let client = self.client;
        // 构建url用于签名使用;签名使用的url的host在构建签名的时候用不到的，理论上可以是任意值，这里用endpoint
        // 此api不涉及bucket，url使用https://endpoint
        let request_url =
            Url::parse_with_params(&format!("https://{}", client.endpoint), query_map).unwrap();

        let mut request_header_map = HashMap::with_capacity(1);
        if let Some(s) = self.x_oss_resource_group_id {
            request_header_map.insert("x-oss-resource-group-id".to_owned(), s.to_owned());
        }

        let header_map = super::utils::get_request_header_with_bucket_region(
            client,
            request_header_map,
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
    pub fn list_buckets(&self) -> ListBucketsBuilder<'_> {
        ListBuckets::builder(self)
    }
}
