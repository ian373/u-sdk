use serde::{Deserialize, Serialize};

// region    --- translate

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct TransRespCheckPart {
    pub message: Option<String>,
    // pub request_id: String,
    pub data: Option<TransResponseDataPart>,
    pub code: String,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct TranslateQuery {
    pub format_type: String,
    pub source_language: String,
    pub target_language: String,
    pub source_text: String,
    pub scene: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct TransResponseDataPart {
    pub word_count: String,
    pub translated: String,
    pub detected_language: Option<String>,
}

// endregion --- translate

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct GetDetectLanguageResp {
    pub detected_language: String,
}
