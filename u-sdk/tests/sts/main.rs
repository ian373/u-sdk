#![cfg(feature = "sts")]

use serde::Deserialize;
use u_sdk::sts;
use u_sdk::sts::ram_policy::{Effect, Policy, Statement};

#[derive(Deserialize, Debug)]
pub struct STSConfig {
    pub access_key_id: String,
    pub access_key_secret: String,
}

impl STSConfig {
    pub fn get_conf() -> Self {
        let file_str = std::fs::read_to_string("tests/sts/config.toml").unwrap();
        toml::from_str(&file_str).unwrap()
    }
}

fn get_sts_client() -> sts::Client {
    let conf = STSConfig::get_conf();
    sts::Client::builder()
        .access_key_id(conf.access_key_id)
        .access_key_secret(conf.access_key_secret)
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
