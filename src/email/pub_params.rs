pub struct PubReqParams {
    pub format: Option<String>,
    pub version: String,
    pub access_key_id: String,
    pub signature: String,
    pub signature_method: String,
    pub timestamp: String,
    pub signature_nonce: String,
    pub region_id: String,
}
