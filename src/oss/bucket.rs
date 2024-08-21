//! 只实现了小部分API
//!
//! [阿里云API文档](https://help.aliyun.com/zh/oss/developer-reference/bucket-operations/)

use super::sign_v4::{HTTPVerb, SignV4Param};
use super::utils::{handle_response_status, into_request_header, SerializeToHashMap};
use super::OSSClient;
use crate::error::Error;
use crate::utils::common::gmt_format;

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use url::Url;

// region:    --- put bucket
#[serde_with::skip_serializing_none]
#[derive(Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct PutBucketHeader<'a> {
    /// 为`None`时该请求头值默认为`private`
    pub x_oss_acl: Option<&'a str>,
    pub x_oss_resource_group_id: Option<&'a str>,
}

impl SerializeToHashMap for PutBucketHeader<'_> {}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct CreateBucketConfiguration<'a> {
    /// 默认为`Standard`
    pub storage_class: Option<&'a str>,
    /// 默认为`LRS`
    pub data_redundancy_type: Option<&'a str>,
}
// endregion: --- put bucket

// region:    --- list objects v2
/// `list-type`将自动设为2
#[serde_as]
#[serde_with::skip_serializing_none]
#[derive(Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct ListObjectsV2Query<'a> {
    // list-type: 2
    pub delimiter: Option<&'a str>,
    pub start_after: Option<&'a str>,
    pub continuation_token: Option<&'a str>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub max_keys: Option<u16>,
    pub prefix: Option<&'a str>,
    pub encoding_type: Option<&'a str>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub fetch_owner: Option<bool>,
}
impl SerializeToHashMap for ListObjectsV2Query<'_> {}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ListBucketResult {
    pub contents: Option<Vec<Content>>,
    pub common_prefixes: Option<CommonPrefixes>,
    pub delimiter: String,
    pub encoding_type: Option<String>,
    pub is_truncated: bool,
    pub start_after: Option<String>,
    pub max_keys: u16,
    pub name: String,
    pub prefix: String,
    pub continuation_token: Option<u32>,
    pub key_count: u32,
    pub next_continuation_token: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct CommonPrefixes {
    pub prefix: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Content {
    pub owner: Option<Owner>,
    pub e_tag: String,
    pub key: String,
    pub last_modified: String,
    pub size: u32,
    pub storage_class: String,
    pub restore_info: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Owner {
    pub display_name: String,
    #[serde(rename = "ID")]
    pub id: String,
}
// endregion: --- list objects v2

// region:    --- get bucket info
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct BucketInfo {
    pub bucket: Bucket,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Bucket {
    pub creation_date: String,
    pub extranet_endpoint: String,
    pub intranet_endpoint: String,
    pub location: String,
    pub storage_class: String,
    pub name: String,
    pub resource_group_id: String,
    pub owner: Owner,
    pub access_control_list: AccessControlList,
    pub data_redundancy_type: String,
    pub cross_region_replication: String,
    pub transfer_acceleration: String,
    pub access_monitor: String,
    pub bucket_policy: BucketPolicy,
    pub comment: String,
    pub server_side_encryption_rule: ServerSideEncryptionRule,
}

#[derive(Deserialize, Debug)]
pub struct ServerSideEncryptionRule {
    #[serde(rename = "SSEAlgorithm")]
    pub ssealgorithm: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct AccessControlList {
    pub grant: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct BucketPolicy {
    pub log_bucket: String,
    pub log_prefix: String,
}
// endregion: --- get bucket info

// xml数据为："<LocationConstraint>oss-cn-hangzhou</LocationConstraint>"，
// 这种情况下使用xml反序列化比较特殊，写法得类似于下面这样：
#[derive(Deserialize)]
struct LocationConstraint {
    #[serde(rename = "$text")]
    pub field: String,
}

// region:    --- get bucket stat
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct BucketStat {
    pub storage: u64,
    pub object_count: u32,
    pub multipart_upload_count: u32,
    pub live_channel_count: u32,
    pub last_modified_time: u64,
    pub standard_storage: u64,
    pub standard_object_count: u32,
    pub infrequent_access_storage: u64,
    pub infrequent_access_real_storage: u64,
    pub infrequent_access_object_count: u32,
    pub archive_storage: u64,
    pub archive_real_storage: u64,
    pub archive_object_count: u32,
    pub cold_archive_storage: u64,
    pub cold_archive_real_storage: u64,
    pub cold_archive_object_count: u32,
}
// endregion: --- get bucket stat

impl OSSClient {
    pub async fn put_bucket(
        &self,
        bucket_name: &str,
        endpoint: &str,
        x_header: Option<PutBucketHeader<'_>>,
        bucket_conf: Option<CreateBucketConfiguration<'_>>,
    ) -> Result<(), Error> {
        let request_url = Url::parse(&format!("https://{}.{}", bucket_name, endpoint)).unwrap();
        let mut canonical_header = BTreeMap::new();
        let put_bucket_map = if let Some(h) = x_header {
            h.serialize_to_hashmap()?
        } else {
            HashMap::with_capacity(0)
        };
        canonical_header.extend(put_bucket_map.iter().map(|(k, v)| (k.as_str(), v.as_str())));
        canonical_header.insert("x-oss-content-sha256", "UNSIGNED-PAYLOAD");
        canonical_header.insert("host", request_url.host_str().unwrap());

        let mut additional_header = BTreeSet::new();
        additional_header.insert("host");
        let now = time::OffsetDateTime::now_utc();
        let sign_v4_param = SignV4Param {
            signing_region: &self.region,
            http_verb: HTTPVerb::Put,
            uri: &request_url,
            bucket: Some(bucket_name),
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

        let rq_xml = quick_xml::se::to_string(&bucket_conf)?;
        // println!("rq_xml: {}", rq_xml);
        let resp = self
            .http_client
            .put(request_url)
            .headers(header_map)
            .body(rq_xml)
            .send()
            .await?;
        let _ = handle_response_status(resp).await?;

        Ok(())
    }

    pub async fn list_objects_v2(
        &self,
        query_params: ListObjectsV2Query<'_>,
    ) -> Result<ListBucketResult, Error> {
        let mut query_map = query_params.serialize_to_hashmap()?;

        // 添加固定的query参数
        query_map.insert("list-type".to_owned(), "2".to_owned());
        let sign_url = Url::parse_with_params(
            &format!("https://{}.{}/", self.bucket, self.endpoint),
            query_map,
        )
        .unwrap();

        let mut canonical_header = BTreeMap::new();
        canonical_header.insert("x-oss-content-sha256", "UNSIGNED-PAYLOAD");
        canonical_header.insert("host", sign_url.host_str().unwrap());

        let mut additional_header = BTreeSet::new();
        additional_header.insert("host");
        let now = time::OffsetDateTime::now_utc();
        let sign_v4_param = SignV4Param {
            signing_region: &self.region,
            http_verb: HTTPVerb::Get,
            uri: &sign_url,
            bucket: Some(&self.bucket),
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
            .get(sign_url)
            .headers(header_map)
            .send()
            .await?;

        let text = handle_response_status(resp).await?;
        // println!("text: {}", text);
        let res = quick_xml::de::from_str(&text)?;

        Ok(res)
    }

    /// - `bucket`: 如果为`None`，则获取[`OSSClient`]中的`bucket`信息，否则获取指定的`bucket`的信息
    pub async fn get_bucket_info(&self) -> Result<BucketInfo, Error> {
        let request_url = Url::parse_with_params(
            &format!("https://{}.{}", self.bucket, self.endpoint),
            [("bucketInfo", "")],
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
            bucket: Some(&self.bucket),
            header_map: &canonical_header,
            additional_header: Some(&additional_header),
            date_time: &now,
        };
        let auth = self.sign_v4(sign_v4_param);
        let mut header = canonical_header.into_iter().collect::<HashMap<_, _>>();
        header.insert("Authorization", &auth);
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

    /// - `other_bucket`: 如果为`None`，则获取[`OSSClient`]中的`bucket`信息，否则获取`other_bucket`的信息
    pub async fn get_bucket_location(&self) -> Result<String, Error> {
        let request_url = Url::parse_with_params(
            &format!("https://{}.{}", self.bucket, self.endpoint),
            [("location", "")],
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
            bucket: Some(&self.bucket),
            header_map: &canonical_header,
            additional_header: Some(&additional_header),
            date_time: &now,
        };
        let auth = self.sign_v4(sign_v4_param);
        let mut header = canonical_header.into_iter().collect::<HashMap<_, _>>();
        header.insert("Authorization", &auth);
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
        let res: LocationConstraint = quick_xml::de::from_str(&text)?;
        Ok(res.field)
    }

    /// - `other_bucket`: 如果为`None`，则获取[`OSSClient`]中的`bucket`信息，否则获取`other_bucket`的信息
    pub async fn get_bucket_stat(&self) -> Result<BucketStat, Error> {
        let request_url = Url::parse_with_params(
            &format!("https://{}.{}", self.bucket, self.endpoint),
            [("stat", "")],
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
            bucket: Some(&self.bucket),
            header_map: &canonical_header,
            additional_header: Some(&additional_header),
            date_time: &now,
        };
        let auth = self.sign_v4(sign_v4_param);
        let mut header = canonical_header.into_iter().collect::<HashMap<_, _>>();
        header.insert("Authorization", &auth);
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
