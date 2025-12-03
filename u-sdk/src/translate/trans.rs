use super::Client;
use super::Error;
use super::types_rs::*;
use crate::translate::utils::parse_json_response;
use std::collections::HashMap;
use u_sdk_common::helper::into_header_map;
use u_sdk_common::open_api_sign::{SignParams, get_openapi_request_header};

impl Client {
    /// 机器翻译-通用版和专业版
    ///
    /// 注意事项:
    /// 1. QPS限制50
    /// 2. 字符长度有上限，且需要自己检查长度
    pub fn translate(&self) -> TranslateBuilder<'_> {
        Translate::builder(self)
    }

    /// > <a href="https://help.aliyun.com/zh/machine-translation/developer-reference/api-alimt-2018-10-12-translategeneral" target="_blank">api文档地址</a>
    ///
    /// 注意：使用翻译的不同的api，需要在控制台开启相应的服务
    pub fn get_detect_language(&self) -> GetDetectLanguageBuilder<'_> {
        GetDetectLanguage::builder(self)
    }
}

impl Translate<'_> {
    pub async fn send(&self) -> Result<TranslateResponse, Error> {
        let client = self.client;
        let creds = client.credentials_provider.load().await?;

        let mut sign_params = SignParams {
            req_method: "GET",
            host: &client.host,
            query_map: self,
            x_acs_action: "TranslateGeneral",
            x_acs_version: "2018-10-12",
            x_acs_security_token: creds.sts_security_token.as_deref(),
            request_body: None,
            style: &client.style,
        };
        if self.scene != "general" {
            sign_params.x_acs_action = "Translate";
        }

        let (common_headers, url_) =
            get_openapi_request_header(&creds.access_key_secret, &creds.access_key_id, sign_params)
                .map_err(|e| Error::Common(format!("get_common_headers error: {}", e)))?;

        let header_map = into_header_map(common_headers);
        let resp = client
            .http_client
            .get(url_)
            .headers(header_map)
            .send()
            .await?;

        let res = parse_json_response(resp).await?;
        Ok(res)
    }
}

impl GetDetectLanguage<'_> {
    pub async fn send(&self) -> Result<String, Error> {
        let client = self.client;
        let creds = client.credentials_provider.load().await?;

        let mut query_map = HashMap::new();
        query_map.insert("SourceText".to_owned(), self.source_text.to_owned());

        let sign_params = SignParams {
            req_method: "GET",
            host: &client.host,
            query_map: &query_map,
            x_acs_action: "GetDetectLanguage",
            x_acs_version: "2018-10-12",
            x_acs_security_token: creds.sts_security_token.as_deref(),
            request_body: None,
            style: &client.style,
        };

        let (common_headers, url_) =
            get_openapi_request_header(&creds.access_key_secret, &creds.access_key_id, sign_params)
                .map_err(|e| Error::Common(format!("get_common_headers error: {}", e)))?;

        let header_map = into_header_map(common_headers);
        let resp = client
            .http_client
            .get(url_)
            .headers(header_map)
            .send()
            .await?;

        let res = parse_json_response::<GetDetectLanguageResp>(resp).await?;
        Ok(res.detected_language)
    }
}
