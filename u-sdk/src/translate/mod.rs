//! 阿里云机器翻译sdk

use bon::bon;

mod error;
pub use error::Error;

mod trans;
mod types_rs;
mod utils;

pub use types_rs::*;
use u_sdk_common::open_api_sign::OpenApiStyle;

pub struct Client {
    access_key_id: String,
    access_key_secret: String,
    http_client: reqwest::Client,
    host: String,
    style: OpenApiStyle,
}

#[bon]
impl Client {
    #[builder(on(String, into))]
    pub fn new(access_key_id: String, access_key_secret: String, host: String) -> Self {
        Self {
            access_key_id,
            access_key_secret,
            http_client: reqwest::Client::new(),
            host,
            style: OpenApiStyle::RPC,
        }
    }
}
