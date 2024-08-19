use super::utils::into_request_header;
use super::OSSClient;
use crate::error::Error;
use crate::oss::sign_v4::{sign_v4, HTTPVerb};
use crate::utils::common::gmt_format;

use serde::{Deserialize, Serialize, Serializer};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use url::Url;

//region ListBucketsQueryParams
#[serde_with::skip_serializing_none]
#[derive(Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct ListBucketsQueryParams<'a> {
    pub prefix: Option<&'a str>,
    pub marker: Option<&'a str>,
    #[serde(serialize_with = "serialize_option_u16_as_string")]
    pub max_keys: Option<u16>,
}
fn serialize_option_u16_as_string<S>(value: &Option<u16>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        Some(v) => serializer.serialize_some(&v.to_string()),
        None => serializer.serialize_none(),
    }
}

impl ListBucketsQueryParams<'_> {
    pub(crate) fn into_hashmap(self) -> HashMap<String, String> {
        serde_json::from_value::<HashMap<String, String>>(serde_json::to_value(self).unwrap())
            .unwrap()
    }
}
//endregion

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
    pub bucket: Option<Vec<Bucket>>,
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
        query_params: Option<ListBucketsQueryParams<'_>>,
    ) -> Result<ListAllMyBucketsResult, Error> {
        let query_map = if let Some(query_params) = query_params {
            query_params.into_hashmap()
        } else {
            HashMap::with_capacity(0)
        };
        // println!("query_map: {:?}", query_map);

        // 此api不涉及bucket，url使用https://endpoint
        let url = Url::parse_with_params(&format!("https://{}", self.endpoint), query_map).unwrap();
        let mut canonical_header = BTreeMap::new();
        canonical_header.insert("x-oss-content-sha256", "UNSIGNED-PAYLOAD");
        canonical_header.insert("host", url.host_str().unwrap());
        if let Some(s) = x_oss_resource_group_id {
            canonical_header.insert("x-oss-resource-group-id", s);
        }
        let mut additional_header = BTreeSet::new();
        additional_header.insert("host");
        let now = time::OffsetDateTime::now_utc();
        let auth = sign_v4(
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
        header.insert("Authorization", &auth);
        let gmt = gmt_format(now);
        header.insert("Date", &gmt);
        let header_map = into_request_header(header);

        let resp = self.http_client.get(url).headers(header_map).send().await?;

        let text = resp.text().await?;
        // println!("text: {}", text);
        let res = quick_xml::de::from_str(&text)?;

        Ok(res)
    }
}
