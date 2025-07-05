//! 文本翻译

pub mod trans;
pub mod types_rs;

pub struct TransClient {
    access_key_id: String,
    access_key_secret: String,
    http_client: reqwest::Client,
    host: String,
}

impl TransClient {
    pub fn new(access_key_id: String, access_key_secret: String, host: String) -> Self {
        Self {
            access_key_id,
            access_key_secret,
            http_client: reqwest::Client::new(),
            host,
        }
    }
}
