use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::utils::{get_uuid, now_iso8601, sign_params};
use super::EmailSdk;

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

pub struct SingleSendEmailSuccessResponse {
    pub env_id: String,
    pub request_id: String,
}

impl EmailSdk {
    pub fn single_send_email(&self, api_params: &SingleSendEmailParams) {
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
            .send();
        match a {
            Ok(resp) => {
                println!("{:?}", resp.text().unwrap())
            }
            Err(e) => {
                println!("{:?}", e)
            }
        }
    }
}
