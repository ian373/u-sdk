use super::Error;
use super::utils::{parse_json_response, sign_params};
use super::{BASE_URL, Client};
use u_sdk_common::helper::now_iso8601;

use bon::Builder;
use serde::{Deserialize, Serialize};

//region response
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct QueryDomainByParamResult {
    pub page_number: u32,
    pub page_size: u32,
    pub request_id: String,
    pub total_count: u32,
    #[serde(rename = "data")]
    pub data: Data,
}

#[derive(Deserialize, Debug)]
pub struct Data {
    pub domain: Vec<PerInfo>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct PerInfo {
    pub cname_auth_status: u8,
    pub confirm_status: u8,
    pub create_time: String,
    pub domain_id: u32,
    pub domain_name: String,
    pub domain_status: u8,
    pub icp_status: u8,
    pub mx_auth_status: u8,
    pub spf_auth_status: u8,
    pub utc_create_time: u64,
    pub domain_record: String,
}
//endregion

#[derive(Builder, Serialize)]
#[builder(on(String, into))]
#[serde(rename_all = "PascalCase")]
pub struct QueryDomainByParam<'a> {
    #[builder(start_fn)]
    #[serde(skip_serializing)]
    client: &'a Client,

    #[serde(skip_serializing_if = "Option::is_none")]
    key_word: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    page_no: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    page_size: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<u8>,
}

impl Client {
    pub fn query_domain_by_param(&self) -> QueryDomainByParamBuilder<'_> {
        QueryDomainByParam::builder(self)
    }
}

impl QueryDomainByParam<'_> {
    pub async fn send(&self) -> Result<QueryDomainByParamResult, Error> {
        let mut map = self.client.known_params.clone();
        map.insert("Timestamp".to_owned(), now_iso8601());
        map.insert(
            "SignatureNonce".to_owned(),
            uuid::Uuid::new_v4().to_string(),
        );

        let mut params_map = serde_json::from_value(serde_json::to_value(self).unwrap()).unwrap();
        map.append(&mut params_map);

        map.insert("Action".to_owned(), "QueryDomainByParam".to_owned());
        let signature = sign_params(&map, &self.client.access_key_secret);
        map.insert("Signature".to_owned(), signature);

        let resp = self
            .client
            .http_client
            .post(BASE_URL)
            .form(&map)
            .send()
            .await?;

        let resp = parse_json_response(resp).await?;
        Ok(resp)
    }
}
