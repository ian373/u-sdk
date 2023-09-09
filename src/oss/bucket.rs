//! 只实现了小部分API

use super::utils::{now_gmt, sign_authorization};
use super::OSSClient;
use crate::error::Error;

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use url::Url;

// region:    --- put bucket
#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct PutBucketHeader<'a> {
    /// 为`None`时该请求头值默认为`private`
    pub x_oss_acl: Option<&'a str>,
    pub x_oss_resource_group_id: Option<&'a str>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]

pub struct CreateBucketConfiguration<'a> {
    /// 默认为`Standard`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_class: Option<&'a str>,
    /// 默认为`LRS`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_redundancy_type: Option<&'a str>,
}
// endregion: --- put bucket

// region:    --- list objects v2
/// `list-type`将自动设为2
#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct ListObjectsV2Query<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delimiter: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_after: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continuation_token: Option<&'a str>,
    // 最好改为u16类型
    /// `u16`类型
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_keys: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding_type: Option<&'a str>,
    /// `bool`类型
    #[serde(skip_serializing_if = "Option::is_none")]
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

impl OSSClient {
    pub async fn put_bucket(
        &self,
        x_header: PutBucketHeader<'_>,
        params: CreateBucketConfiguration<'_>,
        endpoint: &str,
        bucket_name: &str,
    ) -> Result<(), Error> {
        let mut oss_header_map = BTreeMap::new();
        if let Some(s) = x_header.x_oss_acl {
            oss_header_map.insert("x-oss-acl".to_owned(), s.to_owned());
        }
        if let Some(s) = x_header.x_oss_resource_group_id {
            oss_header_map.insert("x-oss-resource-group-id".to_owned(), s.to_owned());
        }

        let now_gmt = now_gmt();
        let authorization = sign_authorization(
            &self.access_key_id,
            &self.access_key_secret,
            "PUT",
            None,
            None,
            &now_gmt,
            Some(&oss_header_map),
            Some(bucket_name),
            None,
        );

        let mut header_map = HashMap::new();
        header_map.extend(oss_header_map);

        let mut common_header = self.get_common_header_map(&authorization, None, None, &now_gmt);
        common_header.insert("Host".to_owned(), format!("{}.{}", bucket_name, endpoint));

        header_map.extend(common_header);

        let header_map: HeaderMap = header_map
            .iter()
            .map(|(k, v)| {
                let name = HeaderName::from_bytes(k.as_bytes()).unwrap();
                let value = HeaderValue::from_bytes(v.as_bytes()).unwrap();
                (name, value)
            })
            .collect();

        // let rq_xml = if params.storage_class.is_none() && params.data_redundancy_type.is_none() {
        //     "".to_owned()
        // } else {
        //     quick_xml::se::to_string(&params).unwrap()
        // };
        // 理论上当params两个属性值都为None时应该rq_xml应该为""，但是此时解析结果为：<CreateBucketConfiguration/>也不影响请求
        let rq_xml = quick_xml::se::to_string(&params).unwrap();
        // println!("rq_xml: {}", rq_xml);
        let resp = self
            .http_client
            .put(format!("https://{}.{}", bucket_name, endpoint))
            .headers(header_map)
            .body(rq_xml)
            .send()
            .await?;

        if resp.status() != StatusCode::OK {
            return Err(Error::StatusCodeNot200Resp(resp));
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

        let header_map: HeaderMap = common_header
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
        // println!("text: {}", text);
        let res = quick_xml::de::from_str(&text).map_err(|e| Error::XMLDeError {
            source: e,
            origin_text: text,
        })?;

        Ok(res)
    }

    /// - `other_bucket`: 如果为`None`，则获取[`OSSClient`]中的`bucket`信息，否则获取`other_bucket`的信息
    pub async fn get_bucket_info(&self, other_bucket: Option<&str>) -> Result<BucketInfo, Error> {
        let now_gmt = now_gmt();
        let bucket_name = if let Some(b) = other_bucket {
            b
        } else {
            &self.bucket
        };
        let url = Url::parse_with_params(
            &format!("https://{}.{}", bucket_name, self.endpoint),
            [("bucketInfo", "")],
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
            Some("?bucketInfo"),
        );

        let mut common_header = self.get_common_header_map(&authorization, None, None, &now_gmt);
        common_header.insert(
            "Host".to_owned(),
            format!("{}.{}", bucket_name, self.endpoint),
        );

        let header_map: HeaderMap = common_header
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
        // println!("resp_text:\n{}", text);
        let res = quick_xml::de::from_str(&text).map_err(|e| Error::XMLDeError {
            source: e,
            origin_text: text,
        })?;

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

        let header_map: HeaderMap = common_header
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
        let res: LocationConstraint =
            quick_xml::de::from_str(&text).map_err(|e| Error::XMLDeError {
                source: e,
                origin_text: text,
            })?;
        Ok(res.field)
    }
}
