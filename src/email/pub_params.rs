use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PubReqParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    pub version: String,
    pub access_key_id: String,
    pub signature_method: String,
    pub timestamp: String,
    pub signature_version: String,
    pub signature_nonce: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region_id: Option<String>,
}
