use super::Client;
use super::ram_policy::Policy;
use bon::Builder;
use serde::{Deserialize, Serialize, Serializer};

/// [AssumRole API](https://help.aliyun.com/zh/ram/developer-reference/api-sts-2015-04-01-assumerole)
#[serde_with::skip_serializing_none]
#[derive(Serialize, Builder)]
#[serde(rename_all = "PascalCase")]
pub struct AssumeRole<'a> {
    #[serde(skip_serializing)]
    #[builder(start_fn)]
    pub(crate) client: &'a Client,
    duration_seconds: Option<u32>,
    // 这个字段是String类型而不是需要flatten的结构体
    #[serde(serialize_with = "policy_as_string")]
    policy: Option<Policy>,
    role_arn: &'a str,
    role_session_name: &'a str,
    external_id: Option<&'a str>,
    source_identity: Option<&'a str>,
    // sts
    #[serde(skip_serializing)]
    pub(crate) sts_security_token: Option<&'a str>,
}

fn policy_as_string<S>(opt: &Option<Policy>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match opt {
        Some(policy) => {
            let s = serde_json::to_string(policy).map_err(serde::ser::Error::custom)?;
            serializer.serialize_str(&s)
        }
        None => serializer.serialize_none(),
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct AssumeRoleResponse {
    pub request_id: String,
    pub assumed_role_user: AssumedRoleUser,
    pub credentials: Credentials,
    pub source_identity: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct AssumedRoleUser {
    pub assumed_role_id: String,
    pub arn: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Credentials {
    pub security_token: String,
    pub expiration: String,
    pub access_key_id: String,
    pub access_key_secret: String,
}
