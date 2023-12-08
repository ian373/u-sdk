use super::open_api_sign::{get_common_headers, SignParams};
use super::TransClient;
use crate::error::Error;
use crate::oss::utils::into_header_map;

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

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

// region    --- general translate response
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct GeneralTransSuccessRespPart {
    // pub request_id: String,
    pub data: GTResponseDataPart,
    // pub code: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct GTResponseDataPart {
    // pub word_count: String,
    pub translated: String,
}
// endregion --- general translate response

impl TransClient {
    /// - [api文档地址](https://help.aliyun.com/zh/machine-translation/developer-reference/api-alimt-2018-10-12-translategeneral)
    ///
    /// 注意事项
    /// 1. QPS限制50
    /// 2. 字符长度上限是5000字符，
    pub async fn general_translate(&self, query: GeneralTranslateQuery) -> Result<String, Error> {
        if query.source_text.len() > 5000 {
            return Err(Error::CommonError("字符长度上限是5000字符".to_owned()));
        }

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
        let (common_headers, url_) =
            get_common_headers(&self.access_key_secret, &self.access_key_id, sign_params);

        let header_map = into_header_map(common_headers);
        let resp = self
            .http_client
            .get(url_)
            .headers(header_map)
            .send()
            .await?;
        if resp.status() != StatusCode::OK {
            return Err(Error::StatusCodeNot200Resp(resp));
        }
        let res = resp.json::<GeneralTransSuccessRespPart>().await?;
        Ok(res.data.translated)
    }
}
