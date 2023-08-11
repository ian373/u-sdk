//! ali cloud email sdk

pub(crate) mod utils;

pub mod async_send_email;
pub mod send_email;

use std::collections::BTreeMap;

pub struct EmailSdk {
    // 公共参数固定不变的部分
    known_params: BTreeMap<String, String>,
    access_key_secret: String,
    http_client: reqwest::blocking::Client,
}

impl EmailSdk {
    pub fn new(
        access_key_id: String,
        access_key_secret: String,
        region_id: Option<String>,
    ) -> Self {
        let mut map = BTreeMap::new();
        map.insert("Format".to_string(), "JSON".to_string());
        map.insert("Version".to_string(), "2015-11-23".to_string());
        map.insert("AccessKeyId".to_string(), access_key_id);
        map.insert("SignatureMethod".to_string(), "HMAC-SHA1".to_string());
        map.insert("SignatureVersion".to_string(), "1.0".to_string());

        if let Some(r) = region_id {
            map.insert("RegionId".to_string(), r);
        }

        Self {
            known_params: map,
            access_key_secret,
            http_client: reqwest::blocking::Client::new(),
        }
    }
}
