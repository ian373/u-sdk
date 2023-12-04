//! TranslateGeneral - 机器翻译通用版调用，同步方法

mod open_api_sign;
mod utils;

pub struct TransClient {
    access_key_id: String,
    access_key_secret: String,
    http_client: reqwest::blocking::Client,
    qps: u8,
    max_text_len: u32,
}

impl TransClient {
    pub fn new(
        access_key_id: String,
        access_key_secret: String,
        qps: u8,
        max_text_len: u32,
    ) -> Self {
        Self {
            access_key_id,
            access_key_secret,
            http_client: reqwest::blocking::Client::new(),
            qps,
            max_text_len,
        }
    }
}
