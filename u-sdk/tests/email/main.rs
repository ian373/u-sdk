#![cfg(feature = "email")]

use serde::Deserialize;
use std::sync::Arc;
use u_sdk::credentials::{Credentials, CredentialsProvider};
use u_sdk::email;

#[derive(Deserialize, Debug)]
struct EmailConfig {
    host: String,
    access_key_id: String,
    access_key_secret: String,
    sts_security_token: Option<String>,
}

struct EmailCredsProvider {
    creds: Arc<Credentials>,
}

impl EmailCredsProvider {
    fn new(
        access_key_id: String,
        access_key_secret: String,
        sts_security_token: Option<String>,
    ) -> Self {
        Self {
            creds: Arc::new(Credentials::new(
                access_key_id,
                access_key_secret,
                sts_security_token,
                None,
            )),
        }
    }
}

#[async_trait::async_trait]
impl CredentialsProvider for EmailCredsProvider {
    async fn load(
        &self,
    ) -> Result<Arc<Credentials>, Box<dyn std::error::Error + Send + Sync + 'static>> {
        Ok(Arc::clone(&self.creds))
    }
}

// 获取本地配置信息测试
#[ignore]
fn get_email_client() -> email::Client {
    let conf_str = std::fs::read_to_string("tests/email/config.toml").unwrap();
    let conf = toml::from_str::<EmailConfig>(&conf_str).unwrap();
    let provider = EmailCredsProvider::new(
        conf.access_key_id,
        conf.access_key_secret,
        conf.sts_security_token,
    );
    email::Client::builder()
        .credentials_provider(std::sync::Arc::new(provider))
        .host(conf.host)
        .build()
}

#[tokio::test]
#[ignore]
async fn single_send_email() {
    let client = get_email_client();
    let response = client
        .single_send_email()
        .account_name("noreply@example.com")
        .address_type("1")
        .reply_to_address("false")
        .subject("test")
        .text_body("text body")
        .to_address("example@example.com")
        .build()
        .send()
        .await;

    match response {
        Ok(data) => println!("ok: {:#?}", data),
        Err(e) => println!("error: {:#?}", e),
    }
}

#[tokio::test]
#[ignore]
async fn desc_account_summary_test() {
    let client = get_email_client();

    match client.desc_account_summary().build().send().await {
        Ok(data) => println!("ok: {:#?}", data),
        Err(e) => println!("error: {:#?}", e),
    }
}

#[tokio::test]
#[ignore]
async fn query_domain_by_param_test() {
    let client = get_email_client();

    match client.query_domain_by_param().build().send().await {
        Ok(data) => println!("ok: {:#?}", data),
        Err(e) => println!("error: {:#?}", e),
    }
}

#[tokio::test]
#[ignore]
async fn get_ip_protection_test() {
    let client = get_email_client();

    match client.get_ip_protection().build().send().await {
        Ok(data) => println!("ok: {:#?}", data),
        Err(e) => println!("error: {:#?}", e),
    }
}
