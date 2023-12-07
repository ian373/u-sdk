//! TranslateGeneral - 机器翻译通用版调用，同步方法

pub mod open_api_sign;
pub mod trans;

pub struct TransClient {
    access_key_id: String,
    access_key_secret: String,
    http_client: reqwest::Client,
    qps: u8,
    max_text_len: u32,
    host: String,
}

impl TransClient {
    pub fn new(
        access_key_id: String,
        access_key_secret: String,
        host: String,
        qps: u8,
        max_text_len: u32,
    ) -> Self {
        Self {
            access_key_id,
            access_key_secret,
            http_client: reqwest::Client::new(),
            qps,
            max_text_len,
            host,
        }
    }
}
