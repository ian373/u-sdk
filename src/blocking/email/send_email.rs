use std::collections::BTreeMap;

use super::EmailSdk;
use crate::email::send_email::SingleSendEmailParams;
use crate::email::utils::{get_uuid, sign_params};
use crate::email::BASE_URL;
use crate::utils::date::now_iso8601;

impl EmailSdk {
    pub fn single_send_email(&self, api_params: &SingleSendEmailParams) {
        // 添加剩余的公共参数
        let mut params_map: BTreeMap<String, String> = BTreeMap::new();
        params_map.append(&mut self.known_params.clone());
        params_map.insert("Timestamp".to_owned(), now_iso8601());
        params_map.insert("SignatureNonce".to_owned(), get_uuid());

        // 添加特定api参数
        let mut api_params_map: BTreeMap<String, String> =
            serde_json::from_value(serde_json::to_value(api_params).unwrap()).unwrap();
        params_map.append(&mut api_params_map);
        params_map.insert("Action".to_owned(), "SingleSendMail".to_owned());

        // 计算和添加签名
        let signature = sign_params(&params_map, &self.access_key_secret);
        params_map.insert("Signature".to_owned(), signature);

        let a = self.http_client.post(BASE_URL).form(&params_map).send();
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
