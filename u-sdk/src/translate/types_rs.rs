use crate::translate::Client;
use bon::Builder;
use serde::{Deserialize, Serialize};

// region    --- translate
/// 官方文档[Translate](https://help.aliyun.com/zh/machine-translation/developer-reference/api-alimt-2018-10-12-translate)
#[derive(Builder, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Translate<'a> {
    #[builder(start_fn)]
    #[serde(skip_serializing)]
    pub(crate) client: &'a Client,
    // sts
    #[serde(skip_serializing)]
    pub(crate) sts_security_token: Option<&'a str>,

    // 请求参数
    format_type: &'a str,
    target_language: &'a str,
    source_language: &'a str,
    pub(crate) source_text: &'a str,
    pub(crate) scene: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    context: Option<&'a str>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct TranslateResponse {
    pub message: Option<String>,
    #[serde(rename = "RequestId")]
    pub request_id: String,
    pub data: Option<TranslateData>,
    pub code: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct TranslateData {
    pub word_count: String,
    pub translated: String,
    pub detected_language: Option<String>,
}
// endregion --- translate

// region    --- detect language
#[derive(Builder)]
pub struct GetDetectLanguage<'a> {
    #[builder(start_fn)]
    pub(crate) client: &'a Client,
    pub(crate) source_text: &'a str,
    // sts token，用于签名时使用
    pub(crate) sts_security_token: Option<&'a str>,
}
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct GetDetectLanguageResp {
    pub detected_language: String,
}
// endregion --- detect language
