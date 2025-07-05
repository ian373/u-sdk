use super::Error;
use super::utils::{parse_json_response, sign_params};
use super::{BASE_URL, Client};
use crate::utils::common::{get_uuid, now_iso8601};

use bon::Builder;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

//region response
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct SingleSendEmailResult {
    pub env_id: String,
    pub request_id: String,
}
//endregion

#[derive(Builder, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct SingleSendEmail<'a> {
    #[builder(start_fn)]
    #[serde(skip_serializing)]
    client: &'a Client,

    account_name: &'a str,
    address_type: &'a str,
    reply_to_address: &'a str,
    subject: &'a str,
    to_address: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    click_trace: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    from_alias: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    html_body: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tag_name: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    text_body: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reply_address: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reply_address_alias: Option<&'a str>,
}

impl Client {
    pub fn single_send_email(&self) -> SingleSendEmailBuilder {
        SingleSendEmail::builder(self)
    }
}

impl SingleSendEmail<'_> {
    pub async fn send(&self) -> Result<SingleSendEmailResult, Error> {
        // 添加剩余的公共参数
        let mut params_map = self.client.known_params.clone();
        params_map.insert("Timestamp".to_owned(), now_iso8601());
        params_map.insert("SignatureNonce".to_owned(), get_uuid());

        // 添加特定api参数
        if self.html_body.is_none() && self.text_body.is_none() {
            return Err(Error::Common(
                "one of html_body or text_body must be set".to_owned(),
            ));
        }

        let mut api_params_map: BTreeMap<String, String> =
            serde_json::from_value(serde_json::to_value(self).unwrap()).unwrap();
        params_map.append(&mut api_params_map);
        params_map.insert("Action".to_owned(), "SingleSendMail".to_owned());

        // 计算和添加签名
        let signature = sign_params(&params_map, &self.client.access_key_secret);
        params_map.insert("Signature".to_owned(), signature);

        let resp = self
            .client
            .http_client
            .post(BASE_URL)
            .form(&params_map)
            .send()
            .await
            .map_err(|e| Error::Other(e.into()))?;

        let resp = parse_json_response(resp).await?;
        Ok(resp)
    }
}
