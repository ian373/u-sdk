#![cfg(feature = "translate")]

use serde::Deserialize;
use u_sdk::translate::*;

#[derive(Deserialize, Debug)]
pub struct AliConfig {
    pub access_key_id: String,
    pub access_key_secret: String,
}

impl AliConfig {
    pub fn get_conf() -> Self {
        let file_str = std::fs::read_to_string("tests/translate/config.toml").unwrap();
        toml::from_str(&file_str).unwrap()
    }
}

fn get_trans_client() -> Client {
    let conf = AliConfig::get_conf();
    Client::builder()
        .access_key_id(conf.access_key_id)
        .access_key_secret(conf.access_key_secret)
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
