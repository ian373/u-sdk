//! 邮件推送sdk

pub use self::types_rs::*;
mod account;
pub use account::{DescAccountSummary, DescAccountSummaryBuilder};

mod domain;
pub use domain::{QueryDomainByParam, QueryDomainByParamBuilder};

mod ip_protection;
pub use ip_protection::{GetIpProtection, GetIpProtectionBuilder};

mod send_email;
pub use send_email::{SingleSendEmail, SingleSendEmailBuilder, SingleSendEmailResult};

mod utils;

mod error;
mod types_rs;

pub use error::Error;

use crate::credentials::CredentialsProvider;
use bon::bon;
use std::sync::Arc;
use u_sdk_common::open_api_sign::OpenApiStyle;

pub struct Client {
    credentials_provider: Arc<dyn CredentialsProvider>,
    host: String,
    http_client: reqwest::Client,
    style: OpenApiStyle,
}

#[bon]
impl Client {
    #[builder(on(String, into))]
    pub fn new(credentials_provider: Arc<dyn CredentialsProvider>, host: String) -> Self {
        Self {
            credentials_provider,
            http_client: reqwest::Client::new(),
            host,
            style: OpenApiStyle::RPC,
        }
    }

    pub fn desc_account_summary(&self) -> DescAccountSummaryBuilder<'_> {
        DescAccountSummary::builder(self)
    }

    pub fn query_domain_by_param(&self) -> QueryDomainByParamBuilder<'_> {
        QueryDomainByParam::builder(self)
    }

    pub fn get_ip_protection(&self) -> GetIpProtectionBuilder<'_> {
        GetIpProtection::builder(self)
    }
}
