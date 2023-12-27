use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::utils::{get_uuid, sign_params};
use super::{EmailSdk, BASE_URL};
use crate::error::Error;
use crate::utils::date::now_iso8601;

#[derive(Serialize)]
pub struct APIParams {
    pub key_word: Option<String>,
    pub page_no: Option<u32>,
    pub page_size: Option<u16>,
    pub status: Option<u8>,
}

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

impl EmailSdk {
    pub async fn query_domain_by_param(
        &self,
        api_params: APIParams,
    ) -> Result<QueryDomainByParamResult, Error> {
        let mut params_map: BTreeMap<String, String> = BTreeMap::new();
        params_map.append(&mut self.known_params.clone());
        params_map.insert("Timestamp".to_owned(), now_iso8601());
        params_map.insert("SignatureNonce".to_owned(), get_uuid());

        if api_params.key_word.is_some() {
            params_map.insert(
                "KeyWord".to_owned(),
                api_params.key_word.as_ref().unwrap().to_string(),
            );
        }
        if api_params.page_no.is_some() {
            params_map.insert("PageNo".to_owned(), api_params.page_no.unwrap().to_string());
        }
        if api_params.page_size.is_some() {
            params_map.insert(
                "PageSize".to_owned(),
                api_params.page_size.unwrap().to_string(),
            );
        }
        if api_params.status.is_some() {
            params_map.insert("Status".to_owned(), api_params.status.unwrap().to_string());
        }
        params_map.insert("Action".to_owned(), "QueryDomainByParam".to_owned());

        let signature = sign_params(&params_map, &self.access_key_secret);
        params_map.insert("Signature".to_owned(), signature);

        let resp = self
            .http_client
            .post(BASE_URL)
            .form(&params_map)
            .send()
            .await?;

        if resp.status() == StatusCode::OK {
            Ok(resp.json::<QueryDomainByParamResult>().await.unwrap())
        } else {
            Err(Error::StatusCodeNot200Resp(resp))
        }
    }
}
