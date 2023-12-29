#[cfg(not(feature = "oss"))]
use time::format_description::well_known::iso8601::{
    Config, EncodedConfig, Iso8601, TimePrecision,
};

#[cfg(not(any(feature = "oss", feature = "translate")))]
pub fn get_uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}

#[cfg(not(any(feature = "email", feature = "translate")))]
pub fn now_gmt() -> String {
    use time::format_description::well_known::Rfc2822;
    time::OffsetDateTime::now_utc()
        .format(&Rfc2822)
        .unwrap()
        .replace("+0000", "GMT")
}

#[cfg(not(feature = "oss"))]
pub fn now_iso8601() -> String {
    const ENCODED_CONFIG: EncodedConfig = Config::DEFAULT
        .set_time_precision(TimePrecision::Second {
            decimal_digits: None,
        })
        .encode();

    time::OffsetDateTime::now_utc()
        .format(&Iso8601::<ENCODED_CONFIG>)
        .unwrap()
}

#[cfg(not(feature = "email"))]
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
#[cfg(not(feature = "email"))]
use std::collections::HashMap;

#[cfg(not(feature = "email"))]
pub fn into_header_map(map: HashMap<String, String>) -> HeaderMap {
    map.iter()
        .map(|(k, v)| {
            let name = HeaderName::from_bytes(k.as_bytes()).unwrap();
            let value = HeaderValue::from_bytes(v.as_bytes()).unwrap();
            (name, value)
        })
        .collect()
}

#[cfg(not(feature = "translate"))]
use hmac::{Hmac, Mac};
#[cfg(not(feature = "translate"))]
use sha1::Sha1;
#[cfg(not(feature = "translate"))]
pub fn sign_hmac_sha1(secret: &str, str_to_sign: &str) -> Vec<u8> {
    type HmacSha1 = Hmac<Sha1>;
    let mut mac = HmacSha1::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(str_to_sign.as_bytes());
    mac.finalize().into_bytes().to_vec()
}
