#![cfg(all(feature = "esa", feature = "sts"))]

use serde::Deserialize;
use std::sync::Arc;
use u_sdk::credentials::{Credentials, CredentialsProvider};
use u_sdk::esa;

#[derive(Deserialize, Debug)]
struct Config {
    access_key_id: String,
    access_key_secret: String,
    role_arn: String,
}

struct ESACredsProvider {
    sts_client: u_sdk::sts::Client,
    role_arn: String,
}

impl ESACredsProvider {
    fn new(sts_client: u_sdk::sts::Client, role_arn: String) -> Self {
        Self {
            sts_client,
            role_arn,
        }
    }
}

struct STSCredsProvider {
    creds: Arc<Credentials>,
}

impl STSCredsProvider {
    fn new(access_key_id: String, access_key_secret: String) -> Self {
        Self {
            creds: Arc::new(Credentials::new(
                access_key_id,
                access_key_secret,
                None,
                None,
            )),
        }
    }
}

#[async_trait::async_trait]
impl CredentialsProvider for STSCredsProvider {
    async fn load(
        &self,
    ) -> Result<Arc<Credentials>, Box<dyn std::error::Error + Send + Sync + 'static>> {
        Ok(self.creds.clone())
    }
}

#[async_trait::async_trait]
impl CredentialsProvider for ESACredsProvider {
    async fn load(
        &self,
    ) -> Result<Arc<Credentials>, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let creds = self
            .sts_client
            .assume_role()
            .role_arn(&self.role_arn)
            .role_session_name("usdk-test")
            .duration_seconds(900)
            .build()
            .send()
            .await?
            .credentials;
        Ok(Arc::new(Credentials::new(
            creds.access_key_id,
            creds.access_key_secret,
            Some(creds.security_token),
            None,
        )))
    }
}

fn get_esa_client() -> esa::Client {
    let file_str = std::fs::read_to_string("tests/esa/config.toml").unwrap();
    let conf = toml::from_str::<Config>(&file_str).unwrap();
    let sts_creds_provider = Arc::new(STSCredsProvider::new(
        conf.access_key_id,
        conf.access_key_secret,
    ));
    let sts_client = u_sdk::sts::Client::builder()
        .credentials_provider(sts_creds_provider)
        .host("sts.cn-qingdao.aliyuncs.com")
        .build();
    let esa_creds_provider = Arc::new(ESACredsProvider::new(sts_client, conf.role_arn));
    esa::Client::builder()
        .credentials_provider(esa_creds_provider)
        .host("esa.cn-hangzhou.aliyuncs.com")
        .build()
}

#[tokio::test]
#[ignore]
async fn list_sites_test() {
    let client = get_esa_client();
    let resp = client
        .list_sites()
        .site_name("example.com".to_string())
        .page_number(1)
        .page_size(10)
        .build()
        .send()
        .await;
    println!("ListSites Response:\n{:#?}", resp);
}

#[tokio::test]
#[ignore]
async fn get_origin_protection_test() {
    let client = get_esa_client();
    let resp = client
        .get_origin_protection()
        .site_id(1234567890)
        .build()
        .send()
        .await;

    println!("GetOriginProtection Response:\n{:#?}", resp);
}
