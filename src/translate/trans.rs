use super::types_rs::*;
use super::TransClient;
use crate::error::Error;
use crate::oss::utils::into_header_map;
use crate::utils::open_api_sign::{get_common_headers, SignParams};

use reqwest::StatusCode;
use std::collections::BTreeMap;

/// > <a href="https://help.aliyun.com/zh/machine-translation/developer-reference/api-alimt-2018-10-12-translategeneral" target="_blank">api文档地址</a>
///
/// 注意：使用翻译的不同的api，需要在控制台开启相应的服务
impl TransClient {
    /// 机器翻译-通用版和专业版
    ///
    /// 注意事项:
    /// 1. QPS限制50
    /// 2. 字符长度上限是5000字符，
    pub async fn translate(&self, query: TranslateQuery) -> Result<TransResponseDataPart, Error> {
        if query.source_text.len() > 5000 {
            return Err(Error::CommonError("字符长度上限是5000字符".to_owned()));
        }

        let query_map = serde_json::from_value(serde_json::to_value(&query).unwrap()).unwrap();

        let mut sign_params = SignParams {
            req_method: "GET",
            host: &self.host,
            query_map: &query_map,
            x_headers: None,
            body_bytes: None,
            x_acs_action: "TranslateGeneral",
            x_acs_version: "2018-10-12",
            x_acs_security_token: None,
        };
        if query.scene != "general" {
            sign_params.x_acs_action = "Translate";
        }

        let (common_headers, url_) =
            get_common_headers(&self.access_key_secret, &self.access_key_id, sign_params)?;

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
        let res = resp.json::<TransRespCheckPart>().await?;
        if res.code != "200" || res.data.is_none() {
            Err(Error::CommonError(format!(
                "msg:{}\ncode:{}",
                res.message.unwrap_or("None".to_owned()),
                res.code
            )))
        } else {
            Ok(res.data.unwrap())
        }
    }

    pub async fn get_detect_language(&self, source_text: &str) -> Result<String, Error> {
        if source_text.len() > 5000 {
            return Err(Error::CommonError("字符长度上限是5000字符".to_owned()));
        }

        let mut query_map = BTreeMap::new();
        query_map.insert("SourceText".to_owned(), source_text.to_owned());

        let sign_params = SignParams {
            req_method: "GET",
            host: &self.host,
            query_map: &query_map,
            x_headers: None,
            body_bytes: None,
            x_acs_action: "GetDetectLanguage",
            x_acs_version: "2018-10-12",
            x_acs_security_token: None,
        };

        let (common_headers, url_) =
            get_common_headers(&self.access_key_secret, &self.access_key_id, sign_params)?;

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
        let res = resp.json::<GetDetectLanguageResp>().await?;

        Ok(res.detected_language)
    }
}
