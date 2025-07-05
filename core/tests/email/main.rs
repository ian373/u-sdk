use serde::Deserialize;
use u_sdk::email;

#[derive(Deserialize, Debug)]
pub struct AliConfig {
    pub access_key_id: String,
    pub access_key_secret: String,
    pub account_name: String,
    pub to_address: String,
}

impl AliConfig {
    pub fn get_conf() -> Self {
        let file_str = std::fs::read_to_string("tests/email/config.toml").unwrap();
        let conf = toml::from_str(&file_str).unwrap();

        conf
    }
}

// 获取本地配置信息测试
#[test]
fn get_test_conf() {
    let s = AliConfig::get_conf();
    println!("{:?}", s);
}

#[tokio::test]
async fn single_send_email() {
    let conf = AliConfig::get_conf();
    let client = email::Client::builder()
        .access_key_id(conf.access_key_id)
        .access_key_secret(conf.access_key_secret)
        .build();
    let response = client
        .single_send_email()
        .account_name(&conf.account_name)
        .address_type("1")
        .reply_to_address("false")
        .subject("这是一个异步发送邮件测试")
        .text_body("text body")
        .to_address(&conf.to_address)
        .build()
        .send()
        .await;

    match response {
        Ok(data) => println!("ok: {:?}", data),
        Err(e) => println!("error: {}", e),
    }
}

#[tokio::test]
async fn desc_account_summary_test() {
    let conf = AliConfig::get_conf();
    let client = email::Client::builder()
        .access_key_id(conf.access_key_id)
        .access_key_secret(conf.access_key_secret)
        .build();

    match client.desc_account_summary().build().send().await {
        Ok(data) => println!("ok: {:?}", data),
        Err(e) => println!("error: {}", e),
    }
}

#[tokio::test]
async fn query_domain_by_param_test() {
    let conf = AliConfig::get_conf();
    let client = email::Client::builder()
        .access_key_id(conf.access_key_id)
        .access_key_secret(conf.access_key_secret)
        .build();

    match client.query_domain_by_param().build().send().await {
        Ok(data) => println!("ok: {:?}", data),
        Err(e) => println!("error: {}", e),
    }
}

#[tokio::test]
async fn get_ip_protection_test() {
    let conf = AliConfig::get_conf();
    let client = email::Client::builder()
        .access_key_id(conf.access_key_id)
        .access_key_secret(conf.access_key_secret)
        .build();

    match client.get_ip_protection().build().send().await {
        Ok(data) => println!("ok: {:?}", data),
        Err(e) => println!("error: {}", e),
    }
}
