//! 只实现了小部分API
//!
//! [阿里云API文档](https://help.aliyun.com/zh/oss/developer-reference/bucket-operations/)

use super::utils::{into_request_header, sign_authorization};
use super::OSSClient;
use crate::error::Error;
use crate::oss::sign_v4::{sign_v4, HTTPVerb};
use crate::utils::common::{gmt_format, into_header_map, now_gmt};

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use url::Url;

// region:    --- put bucket
#[derive(Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct PutBucketHeader<'a> {
    /// 为`None`时该请求头值默认为`private`
    pub x_oss_acl: Option<&'a str>,
    pub x_oss_resource_group_id: Option<&'a str>,
}

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
#[serde_with::skip_serializing_none]
#[derive(Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct ListObjectsV2Query<'a> {
    pub delimiter: Option<&'a str>,
    pub start_after: Option<&'a str>,
    pub continuation_token: Option<&'a str>,
    // 最好改为u16类型
    /// `u16`类型
    pub max_keys: Option<&'a str>,
    pub prefix: Option<&'a str>,
    pub encoding_type: Option<&'a str>,
    /// `bool`类型
    pub fetch_owner: Option<&'a str>,
}

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
        let url = Url::parse(&format!("https://{0}.{1}/{0}/", bucket_name, endpoint)).unwrap();
        let mut canonical_header = BTreeMap::new();
        if let Some(h) = x_header {
            if let Some(acl) = h.x_oss_acl {
                canonical_header.insert("x-oss-acl", acl);
            }
            if let Some(gid) = h.x_oss_resource_group_id {
                canonical_header.insert("x-oss-resource-group-id", gid);
            }
        }
        canonical_header.insert("x-oss-content-sha256", "UNSIGNED-PAYLOAD");
        canonical_header.insert("host", url.host_str().unwrap());

        let mut additional_header = BTreeSet::new();
        additional_header.insert("host");
        let now = time::OffsetDateTime::now_utc();
        let authorization = sign_v4(
            &self.region,
            HTTPVerb::Put,
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

        let rq_xml = quick_xml::se::to_string(&bucket_conf)?;
        // println!("rq_xml: {}", rq_xml);
        let resp = self
            .http_client
            .put(format!("https://{}.{}/", bucket_name, endpoint))
            .headers(header_map)
            .body(rq_xml)
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(Error::RequestAPIFailed {
                status: resp.status().to_string(),
                text: resp.text().await?,
            });
        }

        Ok(())
    }

    pub async fn list_objects_v2(
        &self,
        query_params: ListObjectsV2Query<'_>,
    ) -> Result<ListBucketResult, Error> {
        let query_map: HashMap<String, String> =
            serde_json::from_value(serde_json::to_value(query_params).unwrap()).unwrap();

        let mut url = Url::parse_with_params(&self.bucket_url(), query_map).unwrap();
        // 添加固定的query
        url.query_pairs_mut().append_pair("list-type", "2");

        let now_gmt = now_gmt();
        let authorization = sign_authorization(
            &self.access_key_id,
            &self.access_key_secret,
            "GET",
            None,
            None,
            &now_gmt,
            None,
            Some(&self.bucket),
            None,
        );

        let common_header = self.get_common_header_map(&authorization, None, None, &now_gmt);

        let header_map = into_header_map(common_header);

        let resp = self.http_client.get(url).headers(header_map).send().await?;

        let text = resp.text().await?;
        // println!("text: {}", text);
        let res = quick_xml::de::from_str(&text)?;

        Ok(res)
    }

    /// - `bucket`: 如果为`None`，则获取[`OSSClient`]中的`bucket`信息，否则获取指定的`bucket`的信息
    pub async fn get_bucket_info(&self) -> Result<BucketInfo, Error> {
        let url = Url::parse_with_params(
            &format!("https://{0}.{1}/{0}/", self.bucket, self.endpoint),
            [("bucketInfo", "")],
        )
        .unwrap();
        let mut canonical_header = BTreeMap::new();
        canonical_header.insert("x-oss-content-sha256", "UNSIGNED-PAYLOAD");
        canonical_header.insert("host", url.host_str().unwrap());
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

        let resp = self
            .http_client
            .get(format!(
                "https://{}.{}/?bucketInfo",
                self.bucket, self.endpoint
            ))
            .headers(header_map)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(Error::RequestAPIFailed {
                status: resp.status().to_string(),
                text: resp.text().await?,
            });
        }
        let text = resp.text().await?;
        let res = quick_xml::de::from_str(&text)?;

        Ok(res)
    }

    /// - `other_bucket`: 如果为`None`，则获取[`OSSClient`]中的`bucket`信息，否则获取`other_bucket`的信息
    pub async fn get_bucket_location(&self, other_bucket: Option<&str>) -> Result<String, Error> {
        let now_gmt = now_gmt();
        let bucket_name = if let Some(b) = other_bucket {
            b
        } else {
            &self.bucket
        };
        let url = Url::parse_with_params(
            &format!("https://{}.{}", bucket_name, self.endpoint),
            [("location", "")],
        )
        .unwrap();
        let authorization = sign_authorization(
            &self.access_key_id,
            &self.access_key_secret,
            "GET",
            None,
            None,
            &now_gmt,
            None,
            Some(bucket_name),
            // 你可以认为，这个请求其实是请求bucket的一个特殊object
            Some("?location"),
        );

        let mut common_header = self.get_common_header_map(&authorization, None, None, &now_gmt);
        common_header.insert(
            "Host".to_owned(),
            format!("{}.{}", bucket_name, self.endpoint),
        );

        let header_map = into_header_map(common_header);

        let resp = self.http_client.get(url).headers(header_map).send().await?;

        let text = resp.text().await?;
        let res: LocationConstraint = quick_xml::de::from_str(&text)?;
        Ok(res.field)
    }

    /// - `other_bucket`: 如果为`None`，则获取[`OSSClient`]中的`bucket`信息，否则获取`other_bucket`的信息
    pub async fn get_bucket_stat(&self, other_bucket: Option<&str>) -> Result<BucketStat, Error> {
        let now_gmt = now_gmt();
        let bucket_name = if let Some(b) = other_bucket {
            b
        } else {
            &self.bucket
        };
        let url = Url::parse_with_params(
            &format!("https://{}.{}", bucket_name, self.endpoint),
            [("stat", "")],
        )
        .unwrap();
        let authorization = sign_authorization(
            &self.access_key_id,
            &self.access_key_secret,
            "GET",
            None,
            None,
            &now_gmt,
            None,
            Some(bucket_name),
            // 你可以认为，这个请求其实是请求bucket的一个特殊object
            Some("?stat"),
        );

        let mut common_header = self.get_common_header_map(&authorization, None, None, &now_gmt);
        common_header.insert(
            "Host".to_owned(),
            format!("{}.{}", bucket_name, self.endpoint),
        );

        let header_map = into_header_map(common_header);

        let resp = self.http_client.get(url).headers(header_map).send().await?;

        let text = resp.text().await?;
        let res = quick_xml::de::from_str(&text)?;

        Ok(res)
    }
}
