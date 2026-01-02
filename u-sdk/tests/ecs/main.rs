#![cfg(all(feature = "ecs", feature = "sts"))]

use serde::Deserialize;
use std::sync::Arc;
use u_sdk::credentials::{Credentials, CredentialsProvider};
use u_sdk::ecs;

//region client
#[derive(Deserialize, Debug)]
struct Config {
    access_key_id: String,
    access_key_secret: String,
    role_arn: String,
}

struct ECSCredsProvider {
    sts_client: u_sdk::sts::Client,
    role_arn: String,
}

impl ECSCredsProvider {
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
impl CredentialsProvider for ECSCredsProvider {
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

fn get_ecs_client() -> ecs::Client {
    let file_str = std::fs::read_to_string("tests/ecs/config.toml").unwrap();
    let conf = toml::from_str::<Config>(&file_str).unwrap();
    let sts_creds_provider = Arc::new(STSCredsProvider::new(
        conf.access_key_id,
        conf.access_key_secret,
    ));
    let sts_client = u_sdk::sts::Client::builder()
        .credentials_provider(sts_creds_provider)
        .host("sts.cn-qingdao.aliyuncs.com")
        .build();
    let ecs_creds_provider = Arc::new(ECSCredsProvider::new(sts_client, conf.role_arn));
    ecs::Client::builder()
        .credentials_provider(ecs_creds_provider)
        .host("ecs.cn-hangzhou.aliyuncs.com")
        .build()
}
//endregion

#[tokio::test]
#[ignore]
async fn describe_prefix_list_attributes_test() {
    let client = get_ecs_client();
    let resp = client
        .describe_prefix_list_attributes()
        .region_id("cn-hangzhou")
        .prefix_list_id("pl-xxx")
        .build()
        .send()
        .await;

    println!("res:\n{:#?}", resp);
}
