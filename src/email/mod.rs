//! 邮件推送sdk

mod account;
pub use account::{DescAccountSummary, DescAccountSummaryBuilder, DescAccountSummaryResult};

mod domain;
pub use domain::{QueryDomainByParam, QueryDomainByParamBuilder, QueryDomainByParamResult};

mod ip_protection;
pub use ip_protection::{GetIpProtection, GetIpProtectionBuilder, GetIpProtectionResult};

mod send_email;
pub use send_email::{SingleSendEmail, SingleSendEmailBuilder, SingleSendEmailResult};

mod utils;

mod error;
pub use error::Error;

pub(crate) const BASE_URL: &str = "https://dm.aliyuncs.com";

use bon::bon;
use std::collections::BTreeMap;

pub struct Client {
    // 公共参数固定不变的部分
    known_params: BTreeMap<String, String>,
    access_key_secret: String,
    http_client: reqwest::Client,
}

#[bon]
impl Client {
    #[builder(on(String, into))]
    pub fn new(
        access_key_id: String,
        access_key_secret: String,
        region_id: Option<String>,
    ) -> Self {
        let mut map = BTreeMap::new();
        map.insert("Format".to_owned(), "JSON".to_owned());
        map.insert("Version".to_owned(), "2015-11-23".to_owned());
        map.insert("AccessKeyId".to_owned(), access_key_id);
        map.insert("SignatureMethod".to_owned(), "HMAC-SHA1".to_owned());
        map.insert("SignatureVersion".to_owned(), "1.0".to_owned());

        if let Some(r) = region_id {
            map.insert("RegionId".to_owned(), r.into());
        }

        Self {
            known_params: map,
            access_key_secret,
            http_client: reqwest::Client::new(),
        }
    }
}
