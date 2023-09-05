//! oss sdk

use std::collections::HashMap;

pub mod bucket;
pub mod service;

pub(crate) mod utils;

pub struct OSSClient {
    access_key_id: String,
    access_key_secret: String,
    endpoint: String,
    bucket: String,
    http_client: reqwest::Client,
}

impl OSSClient {
    pub fn new(
        access_key_id: String,
        access_key_secret: String,
        endpoint: String,
        bucket: String,
    ) -> Self {
        OSSClient {
            access_key_id,
            access_key_secret,
            endpoint,
            bucket,
            http_client: reqwest::Client::new(),
        }
    }

    fn bucket_url(&self) -> String {
        format!("https://{}.{}", self.bucket, self.endpoint)
    }

    fn endpoint_url(&self) -> String {
        format!("https://{}", self.endpoint)
    }

    fn get_common_header_map(
        &self,
        authorization: &str,
        content_length: Option<&str>,
        content_type: Option<&str>,
        date: &str,
    ) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("Authorization".to_owned(), authorization.to_owned());
        if let Some(s) = content_length {
            map.insert("Content-Length".to_owned(), s.to_owned());
        }
        if let Some(s) = content_type {
            map.insert("Content-Type".to_owned(), s.to_owned());
        }
        map.insert("Date".to_owned(), date.to_owned());
        map.insert(
            "Host".to_owned(),
            format!("{}.{}", self.bucket, self.endpoint),
        );

        map
    }
}
