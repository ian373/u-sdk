//! 只实现了小部分API
//!
//! [阿里云API文档](https://help.aliyun.com/zh/oss/developer-reference/bucket-operations/)

use super::Client;
use super::sign_v4::HTTPVerb;
use super::utils::{
    get_request_header, get_request_header_with_bucket_region, into_request_failed_error,
    parse_xml_response,
};
use crate::oss::Error;
use bon::Builder;
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use std::collections::HashMap;
use url::Url;

// region:    --- put bucket
#[serde_with::skip_serializing_none]
#[derive(Builder, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct PutBucket<'a> {
    #[builder(start_fn)]
    #[serde(skip_serializing)]
    pub(crate) client: &'a Client,
    #[serde(skip_serializing)]
    pub(crate) bucket_name: &'a str,
    // 请求参数
    #[serde(skip_serializing)]
    pub(crate) storage_class: Option<&'a str>,
    #[serde(skip_serializing)]
    pub(crate) data_redundancy_type: Option<&'a str>,
    // header
    x_oss_acl: Option<&'a str>,
    x_oss_resource_group_id: Option<&'a str>,
    x_oss_bucket_tagging: Option<&'a str>,
}

#[serde_with::skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct CreateBucketConfiguration<'a> {
    /// 默认为`Standard`
    storage_class: Option<&'a str>,
    /// 默认为`LRS`
    data_redundancy_type: Option<&'a str>,
}

impl PutBucket<'_> {
    pub async fn send(&self) -> Result<(), Error> {
        let client = self.client;
        let request_url =
            Url::parse(&format!("https://{}.{}", self.bucket_name, client.endpoint)).unwrap();
        let req_header_map = serde_json::from_value(serde_json::to_value(self).unwrap()).unwrap();

        let header = get_request_header_with_bucket_region(
            client,
            req_header_map,
            &request_url,
            HTTPVerb::Put,
            &self.client.region,
            Some(self.bucket_name),
        );

        let req_xml = {
            let create_conf = CreateBucketConfiguration {
                storage_class: self.storage_class,
                data_redundancy_type: self.data_redundancy_type,
            };

            quick_xml::se::to_string(&create_conf).unwrap()
        };

        let resp = client
            .http_client
            .put(request_url)
            .headers(header)
            .body(req_xml)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(into_request_failed_error(resp).await);
        }

        Ok(())
    }
}
// endregion: --- put bucket

// region:    --- list objects v2
/// `list-type`将自动设为2
#[serde_as]
#[serde_with::skip_serializing_none]
#[derive(Builder, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct ListObjectsV2<'a> {
    #[builder(start_fn)]
    #[serde(skip_serializing)]
    pub(crate) client: &'a Client,
    // list-type: 2  固定的，自动添加
    delimiter: Option<&'a str>,
    start_after: Option<&'a str>,
    continuation_token: Option<&'a str>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    max_keys: Option<u16>,
    prefix: Option<&'a str>,
    encoding_type: Option<&'a str>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    fetch_owner: Option<bool>,
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
    pub r#type: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Owner {
    pub display_name: String,
    #[serde(rename = "ID")]
    pub id: String,
}

impl ListObjectsV2<'_> {
    pub async fn send(&self) -> Result<ListBucketResult, Error> {
        let mut query_map: HashMap<String, String> =
            serde_json::from_value(serde_json::to_value(self).unwrap()).unwrap();
        // 添加固定的query参数
        query_map.insert("list-type".to_owned(), "2".to_owned());

        let client = self.client;
        let sign_url = Url::parse_with_params(
            &format!("https://{}.{}/", client.bucket, client.endpoint),
            query_map,
        )
        .unwrap();

        let header = get_request_header(client, HashMap::new(), &sign_url, HTTPVerb::Get);

        let resp = client
            .http_client
            .get(sign_url)
            .headers(header)
            .send()
            .await?;

        let res = parse_xml_response(resp).await?;
        Ok(res)
    }
}
// endregion: --- list objects v2

// region:    --- get bucket info
#[derive(Builder)]
pub struct GetBucketInfo<'a> {
    #[builder(start_fn)]
    pub(crate) client: &'a Client,
    pub(crate) bucket: &'a str,
}

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
    pub versioning: Option<String>,
    pub cross_region_replication: String,
    pub transfer_acceleration: String,
    pub access_monitor: String,
    pub bucket_policy: BucketPolicy,
    pub comment: String,
    pub server_side_encryption_rule: ServerSideEncryptionRule,
    pub block_public_access: bool,
}

#[derive(Deserialize, Debug)]
pub struct ServerSideEncryptionRule {
    #[serde(rename = "SSEAlgorithm")]
    pub sse_algorithm: String,
    pub kms_master_key_id: Option<String>,
    pub kms_data_encryption: Option<String>,
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

impl GetBucketInfo<'_> {
    pub async fn send(&self) -> Result<BucketInfo, Error> {
        let client = self.client;
        let request_url = Url::parse_with_params(
            &format!("https://{}.{}", self.bucket, client.endpoint),
            [("bucketInfo", "")],
        )
        .unwrap();

        let header_map = get_request_header(client, HashMap::new(), &request_url, HTTPVerb::Get);
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
// endregion: --- get bucket info

//region get bucket location
#[derive(Builder)]
pub struct GetBucketLocation<'a> {
    #[builder(start_fn)]
    pub(crate) client: &'a Client,
    pub(crate) bucket: &'a str,
}
// xml数据为："<LocationConstraint>oss-cn-hangzhou</LocationConstraint>"，
// 这种情况下使用xml反序列化比较特殊，写法得类似于下面这样：
#[derive(Deserialize)]
struct LocationConstraint {
    #[serde(rename = "$text")]
    field: String,
}

impl GetBucketLocation<'_> {
    pub async fn send(&self) -> Result<String, Error> {
        let client = self.client;

        let request_url = Url::parse_with_params(
            &format!("https://{}.{}", self.bucket, client.endpoint),
            [("location", "")],
        )
        .unwrap();

        let header_map = get_request_header(client, HashMap::new(), &request_url, HTTPVerb::Get);
        let resp = client
            .http_client
            .get(request_url)
            .headers(header_map)
            .send()
            .await?;

        let res = parse_xml_response::<LocationConstraint>(resp).await?;
        Ok(res.field)
    }
}
//endregion

// region:    --- get bucket stat
#[derive(Builder)]
pub struct GetBucketStat<'a> {
    #[builder(start_fn)]
    pub(crate) client: &'a Client,
    pub(crate) bucket: &'a str,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct BucketStat {
    pub storage: u64,
    pub object_count: u32,
    pub multipart_upload_count: u32,
    pub delete_marker_count: u32,
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
    pub deep_cold_archive_storage: u64,
    pub deep_cold_archive_real_storage: u64,
    pub deep_cold_archive_object_count: u32,
}

impl GetBucketStat<'_> {
    pub async fn send(&self) -> Result<BucketStat, Error> {
        let client = self.client;
        let request_url = Url::parse_with_params(
            &format!("https://{}.{}", self.bucket, client.endpoint),
            [("stat", "")],
        )
        .unwrap();

        let header_map = get_request_header(client, HashMap::new(), &request_url, HTTPVerb::Get);

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
// endregion: --- get bucket stat

impl Client {
    pub fn put_bucket(&self) -> PutBucketBuilder<'_> {
        PutBucket::builder(self)
    }

    pub fn list_objects_v2(&self) -> ListObjectsV2Builder<'_> {
        ListObjectsV2::builder(self)
    }

    pub fn get_bucket_info(&self) -> GetBucketInfoBuilder<'_> {
        GetBucketInfo::builder(self)
    }

    pub fn get_bucket_location(&self) -> GetBucketLocationBuilder<'_> {
        GetBucketLocation::builder(self)
    }

    pub fn get_bucket_stat(&self) -> GetBucketStatBuilder<'_> {
        GetBucketStat::builder(self)
    }
}
