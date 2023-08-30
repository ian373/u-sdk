use reqwest::StatusCode;
use serde::Deserialize;
use std::collections::BTreeMap;

use super::utils::{get_uuid, now_iso8601, sign_params};
use super::{EmailSdk, BASE_URL};
use crate::error::Error;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct DescAccountSummaryResult {
    pub daily_quota: u32,
    pub domains: u32,
    pub enable_times: u32,
    pub mail_addresses: u32,
    pub max_quota_level: u32,
    pub month_quota: u32,
    pub quota_level: u8,
    pub request_id: String,
    pub tags: u32,
    pub templates: u32,
    pub user_status: u8,
}

impl EmailSdk {
    pub async fn desc_account_summary(&self) -> Result<DescAccountSummaryResult, Error> {
        let mut params_map: BTreeMap<String, String> = BTreeMap::new();
        params_map.append(&mut self.known_params.clone());
        params_map.insert("Timestamp".to_owned(), now_iso8601());
        params_map.insert("SignatureNonce".to_owned(), get_uuid());

        params_map.insert("Action".to_owned(), "DescAccountSummary".to_owned());

        let signature = sign_params(&params_map, &self.access_key_secret);
        params_map.insert("Signature".to_owned(), signature);

        let resp = self
            .http_client
            .post(BASE_URL)
            .form(&params_map)
            .send()
            .await?;

        if resp.status() == StatusCode::OK {
            Ok(resp.json::<DescAccountSummaryResult>().await.unwrap())
        } else {
            Err(Error::StatusCodeNot200Resp(resp))
        }
    }
}
