use super::utils::{now_gmt, sign_authorization};
use super::OSSClient;
use crate::error::Error;

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::StatusCode;
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};

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
        println!("rq_xml: {}", rq_xml);
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
}
