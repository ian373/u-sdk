//! ali cloud email sdk

pub mod utils;

pub mod pub_params;
pub mod send_email;

pub struct EmailSdk {
    access_key_id: String,
    access_key_secret: String,
    http_client: reqwest::blocking::Client,
}

impl EmailSdk {
    pub fn new(access_key_id: String, access_key_secret: String) -> Self {
        Self {
            access_key_id,
            access_key_secret,
            http_client: reqwest::blocking::Client::new(),
        }
    }
}
