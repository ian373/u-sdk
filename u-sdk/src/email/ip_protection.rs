use super::Error;
use super::utils::{parse_json_response, sign_params};
use super::{BASE_URL, Client};
use bon::Builder;
use serde::Deserialize;
use std::collections::BTreeMap;
use u_sdk_common::helper::now_iso8601;

//region response
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct GetIpProtectionResult {
    pub ip_protection: String,
    pub request_id: String,
}
//endregion

#[derive(Builder)]
pub struct GetIpProtection<'a> {
    #[builder(start_fn)]
    client: &'a Client,
}

impl Client {
    pub fn get_ip_protection(&self) -> GetIpProtectionBuilder {
        GetIpProtection::builder(self)
    }
}

impl GetIpProtection<'_> {
    pub async fn send(&self) -> Result<GetIpProtectionResult, Error> {
        let mut params_map: BTreeMap<String, String> = BTreeMap::new();
        params_map.append(&mut self.client.known_params.clone());
        params_map.insert("Timestamp".to_owned(), now_iso8601());
        params_map.insert(
            "SignatureNonce".to_owned(),
            uuid::Uuid::new_v4().to_string(),
        );

        params_map.insert("Action".to_owned(), "GetIpProtection".to_owned());

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
