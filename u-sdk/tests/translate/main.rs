#![cfg(feature = "translate")]

use serde::Deserialize;
use std::sync::Arc;
use u_sdk::credentials::{Credentials, CredentialsError, CredentialsProvider};
use u_sdk::translate::*;

#[derive(Deserialize, Debug)]
struct MTConfig {
    pub access_key_id: String,
    pub access_key_secret: String,
    pub sts_security_token: Option<String>,
}

struct MTCredentialsProvider {
    creds: Credentials,
}

impl MTCredentialsProvider {
    fn new(
        access_key_id: String,
        access_key_secret: String,
        sts_security_token: Option<String>,
    ) -> Self {
        Self {
            creds: Credentials::new(access_key_id, access_key_secret, sts_security_token, None),
        }
    }
}

#[async_trait::async_trait]
impl CredentialsProvider for MTCredentialsProvider {
    async fn load(&self) -> Result<Credentials, CredentialsError> {
        Ok(self.creds.clone())
    }
}

fn get_trans_client() -> Client {
    let conf_str = std::fs::read_to_string("tests/translate/config.toml").unwrap();
    let conf = toml::from_str::<MTConfig>(&conf_str).unwrap();
    let provider = MTCredentialsProvider::new(
        conf.access_key_id,
        conf.access_key_secret,
        conf.sts_security_token,
    );
    Client::builder()
        .credentials_provider(Arc::new(provider))
        .host("mt.cn-hangzhou.aliyuncs.com")
        .build()
}

#[tokio::test]
#[ignore]
async fn translate_test() {
    let client = get_trans_client();
    let res = client
        .translate()
        .format_type("text")
        .source_language("auto")
        .target_language("zh")
        .source_text("test first line.\ntest second line.")
        .scene("general")
        .build()
        .send()
        .await;

    match res {
        Ok(s) => println!("[success] res:\n{:#?}", s),
        Err(e) => println!("[error] {:#?}", e),
    }
}

#[tokio::test]
#[ignore]
async fn get_detect_language_test() {
    let client = get_trans_client();
    let res = client
        .get_detect_language()
        .source_text("中文")
        .build()
        .send()
        .await;
    match res {
        Ok(s) => println!("[success] res: {}", s),
        Err(e) => println!("[error] {:#?}", e),
    }
}
