use reqwest::StatusCode;
use serde::Deserialize;
use std::collections::BTreeMap;

use super::utils::{get_uuid, now_iso8601, sign_params};
use super::{EmailSdk, BASE_URL};
use crate::error::Error;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct GetIpProtectionResult {
    pub ip_protection: String,
    pub request_id: String,
}

impl EmailSdk {
    pub async fn get_ip_protection(&self) -> Result<GetIpProtectionResult, Error> {
        let mut params_map: BTreeMap<String, String> = BTreeMap::new();
        params_map.append(&mut self.known_params.clone());
        params_map.insert("Timestamp".to_owned(), now_iso8601());
        params_map.insert("SignatureNonce".to_owned(), get_uuid());

        params_map.insert("Action".to_owned(), "GetIpProtection".to_owned());

        let signature = sign_params(&params_map, &self.access_key_secret);
        params_map.insert("Signature".to_owned(), signature);

        let resp = self
            .http_client
            .post(BASE_URL)
            .form(&params_map)
            .send()
            .await?;

        if resp.status() == StatusCode::OK {
            Ok(resp.json::<GetIpProtectionResult>().await.unwrap())
        } else {
            Err(Error::StatusCodeNot200Resp(resp))
        }
    }
}
