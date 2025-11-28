use super::Client;
use super::ram_policy::Policy;
use bon::Builder;
use serde::{Deserialize, Serialize};

#[serde_with::skip_serializing_none]
#[derive(Serialize, Builder)]
#[serde(rename_all = "PascalCase")]
pub struct AssumeRole<'a> {
    #[serde(skip_serializing)]
    #[builder(start_fn)]
    pub(crate) client: &'a Client,
    duration_seconds: Option<u32>,
    policy: Option<Policy>,
    role_arn: &'a str,
    role_session_name: &'a str,
    external_id: Option<&'a str>,
    source_identity: Option<&'a str>,
    // sts
    #[serde(skip_serializing)]
    pub(crate) sts_security_token: Option<&'a str>,
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
