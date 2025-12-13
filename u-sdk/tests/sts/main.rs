#![cfg(feature = "sts")]

use async_trait::async_trait;
use serde::Deserialize;
use std::sync::Arc;
use u_sdk::credentials::{Credentials, CredentialsProvider};
use u_sdk::sts;
use u_sdk::sts::ram_policy::{Effect, Policy, Statement};

#[derive(Deserialize, Debug)]
pub struct STSConfig {
    pub access_key_id: String,
    pub access_key_secret: String,
    pub sts_security_token: Option<String>,
}

pub struct STSCredsProvider {
    creds: Arc<Credentials>,
}

impl STSCredsProvider {
    pub fn new(
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

#[async_trait]
impl CredentialsProvider for STSCredsProvider {
    async fn load(
        &self,
    ) -> Result<Arc<Credentials>, Box<dyn std::error::Error + Send + Sync + 'static>> {
        Ok(Arc::clone(&self.creds))
    }
}

fn get_sts_client() -> sts::Client {
    let conf_str = std::fs::read_to_string("tests/sts/config.toml").unwrap();
    let conf = toml::from_str::<STSConfig>(&conf_str).unwrap();
    let provider = STSCredsProvider::new(
        conf.access_key_id,
        conf.access_key_secret,
        conf.sts_security_token,
    );

    sts::Client::builder()
        .credentials_provider(Arc::new(provider))
        .host("sts.cn-hangzhou.aliyuncs.com")
        .build()
}

#[tokio::test]
#[ignore]
async fn assume_role_test() {
    let client = get_sts_client();
    let stmt1 = Statement {
        effect: Effect::Allow,
        action: Some("oss:ListObjects".to_owned().into()),
        not_action: None,
        resource: "acs:oss:*:*:app".to_owned().into(),
        condition: None,
    };
    let stmt2 = Statement {
        effect: Effect::Allow,
        action: Some("oss:GetObject".to_owned().into()),
        not_action: None,
        resource: "acs:oss:*:*:app/".to_owned().into(),
        condition: None,
    };
    let policy = Policy::builder()
        .statements([stmt1, stmt2])
        .unwrap()
        .build();
    println!("policy json:\n{}", policy.to_json_string_pretty().unwrap());
    let res = client
        .assume_role()
        .duration_seconds(3600)
        .policy(policy)
        .role_arn("xxx")
        .role_session_name("test-session")
        .build()
        .send()
        .await;

    match res {
        Ok(s) => println!("[success] res:\n{:#?}", s),
        Err(e) => println!("[error] {:#?}", e),
    }
}
