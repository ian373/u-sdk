use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::pub_params::PubReqParams;
use super::utils::sign_params;
use super::EmailSdk;

pub const SINGLE_SEND_EMAIL_BASE_URL: &str = "http://dm.aliyuncs.com";

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SingleSendEmailParams {
    pub account_name: String,
    pub address_type: String,
    pub reply_to_address: String,
    pub subject: String,
    pub to_address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
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
    pub fn single_send_email(&self, pub_params: &PubReqParams, api_params: &SingleSendEmailParams) {
        let mut pub_val = serde_json::to_value(pub_params).unwrap();
        let mut api_val = serde_json::to_value(api_params).unwrap();
        pub_val
            .as_object_mut()
            .unwrap()
            .append(api_val.as_object_mut().unwrap());

        let mut params_map: BTreeMap<String, String> = serde_json::from_value(pub_val).unwrap();
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
