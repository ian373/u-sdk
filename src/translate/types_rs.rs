use serde::{Deserialize, Serialize};

// region    --- general translate
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
// endregion --- general translate
