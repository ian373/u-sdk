use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::utils::{get_uuid, now_iso8601, sign_params};

pub struct EmailSdkAsync {
    known_params: BTreeMap<String, String>,
    access_key_secret: String,
    http_client: reqwest::Client,
}

impl EmailSdkAsync {
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
            http_client: reqwest::Client::new(),
        }
    }
}

const SINGLE_SEND_EMAIL_BASE_URL: &str = "http://dm.aliyuncs.com";

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SingleSendEmailParams {
    pub account_name: String,
    pub address_type: String,
    pub reply_to_address: String,
    pub subject: String,
    pub to_address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub click_trace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_alias: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_address_alias: Option<String>,
}

impl EmailSdkAsync {
    pub async fn single_send_email_async(&self, api_params: &SingleSendEmailParams) {
        // 添加剩余的公共参数
        let mut params_map: BTreeMap<String, String> = BTreeMap::new();
        params_map.append(&mut self.known_params.clone());
        params_map.insert("Timestamp".to_string(), now_iso8601());
        params_map.insert("SignatureNonce".to_string(), get_uuid());

        // 添加特定api参数
        let mut api_params_map: BTreeMap<String, String> =
            serde_json::from_value(serde_json::to_value(api_params).unwrap()).unwrap();
        params_map.append(&mut api_params_map);
        params_map.insert("Action".to_string(), "SingleSendMail".to_string());

        // 计算和添加签名
        let signature = sign_params(&params_map, &self.access_key_secret);
        params_map.insert("Signature".to_owned(), signature);

        let a = self
            .http_client
            .post(SINGLE_SEND_EMAIL_BASE_URL)
            .form(&params_map)
            .send()
            .await;
        match a {
            Ok(resp) => {
                println!("{:?}", resp.text().await.unwrap())
            }
            Err(e) => {
                println!("{:?}", e)
            }
        }
    }
}
