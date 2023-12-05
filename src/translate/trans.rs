use super::open_api_sign::{get_common_headers, SignParams};
use super::TransClient;
use crate::oss::utils::into_header_map;
use serde::Serialize;
// use std::collections::BTreeMap;

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct GeneralTranslateQuery {
    pub format_type: String,
    pub source_language: String,
    pub target_language: String,
    pub source_text: String,
    pub scene: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

impl TransClient {
    pub fn general_translate(&self, query: GeneralTranslateQuery) {
        let query_map = serde_json::from_value(serde_json::to_value(query).unwrap()).unwrap();

        let sign_params = SignParams {
            req_method: "GET",
            host: &self.host,
            query_map: &query_map,
            x_headers: None,
            body_bytes: None,
            x_acs_action: "TranslateGeneral",
            x_acs_version: "2018-10-12",
            x_acs_security_token: None,
        };
        let (common_headers, url_) = get_common_headers(&self.access_key_secret, sign_params);

        let header_map = into_header_map(common_headers);
        let res = self
            .http_client
            .get(url_)
            .headers(header_map)
            .send()
            .unwrap();
        println!("response:\n{}", res.text().unwrap());
    }
}
