use u_sdk::email;
use u_sdk::error::Error;

use serde::Deserialize;

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
    let client = email::EmailSdk::new(conf.access_key_id, conf.access_key_secret, None);
    let params = email::send_email::SingleSendEmailParams {
        account_name: &conf.account_name,
        address_type: "1",
        reply_to_address: "false",
        subject: "这是一个异步发送邮件测试",
        to_address: &conf.to_address,
        click_trace: None,
        from_alias: None,
        html_body: Some("这是一个异步的测试"),
        tag_name: None,
        text_body: None,
        reply_address: None,
        reply_address_alias: None,
    };

    match client.single_send_email(&params).await {
        Ok(data) => {
            println!("ok: {:?}", data);
        }
        Err(e) => match e {
            Error::RequestFailed(e) => {
                println!("error: {:?}", e);
            }
            _ => println!("error: {}", e),
        },
    }
}

#[tokio::test]
async fn desc_account_summary_test() {
    let conf = AliConfig::get_conf();
    let client = email::EmailSdk::new(conf.access_key_id, conf.access_key_secret, None);

    match client.desc_account_summary().await {
        Ok(data) => {
            println!("ok: {:?}", data);
        }
        Err(e) => match e {
            Error::RequestFailed(e) => {
                println!("error: {:?}", e);
            }
            _ => println!("error: {}", e),
        },
    }
}

#[tokio::test]
async fn query_domain_by_param_test() {
    let conf = AliConfig::get_conf();
    let client = email::EmailSdk::new(conf.access_key_id, conf.access_key_secret, None);

    let api_params = email::domain::APIParams {
        key_word: None,
        page_no: None,
        page_size: None,
        status: None,
    };

    match client.query_domain_by_param(api_params).await {
        Ok(data) => {
            println!("ok: {:?}", data);
        }
        Err(e) => match e {
            Error::RequestFailed(e) => {
                println!("error: {:?}", e);
            }
            _ => println!("error: {}", e),
        },
    }
}

#[tokio::test]
async fn get_ip_protection_test() {
    let conf = AliConfig::get_conf();
    let client = email::EmailSdk::new(conf.access_key_id, conf.access_key_secret, None);

    match client.get_ip_protection().await {
        Ok(data) => {
            println!("ok: {:?}", data);
        }
        Err(e) => match e {
            Error::RequestFailed(e) => {
                println!("error: {:?}", e);
            }
            _ => println!("error: {}", e),
        },
    }
}
