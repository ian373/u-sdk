use super::sign_v4::{HTTPVerb, SignV4Param};
use super::utils::{handle_response_status, into_request_header, SerializeToHashMap};
use super::OSSClient;
use crate::error::Error;
use crate::utils::common::gmt_format;

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use url::Url;

//region ListBucketsQueryParams
#[serde_as]
#[serde_with::skip_serializing_none]
#[derive(Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct ListBucketsQueryParams<'a> {
    pub prefix: Option<&'a str>,
    pub marker: Option<&'a str>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub max_keys: Option<u16>,
}

impl SerializeToHashMap for ListBucketsQueryParams<'_> {}
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
        // 构建url的query部分，用于传递url以便签名构建canonical_query_string
        let query_map = if let Some(query_params) = query_params {
            query_params.serialize_to_hashmap()?
        } else {
            HashMap::with_capacity(0)
        };
        // println!("query_map: {:?}", query_map);

        // 构建url用于签名使用;签名使用的url的host在构建签名的时候用不到的，理论上可以是任意值，这里用endpoint
        // 此api不涉及bucket，url使用https://endpoint
        let request_url =
            Url::parse_with_params(&format!("https://{}", self.endpoint), query_map).unwrap();
        // 构建canonical_header，canonical_header包含了公共请求头的部分内容
        let mut canonical_header = BTreeMap::new();
        // x-oss-content-sha256是必须存在且值固定
        canonical_header.insert("x-oss-content-sha256", "UNSIGNED-PAYLOAD");
        // host为addition_header中指定的需要额外添加到签名计算中的参数
        canonical_header.insert("host", request_url.host_str().unwrap());
        if let Some(s) = x_oss_resource_group_id {
            canonical_header.insert("x-oss-resource-group-id", s);
        }
        // 添加host到additional_header，因为canonical_header中把host也参与签名计算了
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
        // 把canonical_header转换为HashMap，用于构建请求头，添加剩余的必须的公共请求头
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
        // println!("text: {}", text);
        let res = quick_xml::de::from_str(&text)?;

        Ok(res)
    }
}
