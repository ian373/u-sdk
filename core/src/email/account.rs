use super::Error;
use super::utils::{parse_json_response, sign_params};
use super::{BASE_URL, Client};
use common_lib::helper::now_iso8601;

use bon::Builder;
use serde::Deserialize;

//region response
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
//endregion

#[derive(Builder)]
pub struct DescAccountSummary<'a> {
    #[builder(start_fn)]
    client: &'a Client,
}

impl Client {
    pub fn desc_account_summary(&self) -> DescAccountSummaryBuilder {
        DescAccountSummary::builder(self)
    }
}

impl DescAccountSummary<'_> {
    pub async fn send(&self) -> Result<DescAccountSummaryResult, Error> {
        let mut params_map = self.client.known_params.clone();

        params_map.insert("Timestamp".to_owned(), now_iso8601());
        params_map.insert(
            "SignatureNonce".to_owned(),
            uuid::Uuid::new_v4().to_string(),
        );
        params_map.insert("Action".to_owned(), "DescAccountSummary".to_owned());

        let signature = sign_params(&params_map, &self.client.access_key_secret);
        params_map.insert("Signature".to_owned(), signature);

        let resp = self
            .client
            .http_client
            .post(BASE_URL)
            .form(&params_map)
            .send()
            .await?;

        let resp = parse_json_response(resp).await?;
        Ok(resp)
    }
}
