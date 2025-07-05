use hmac::{Hmac, Mac};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use sha1::Sha1;
use std::collections::HashMap;
use time::OffsetDateTime;
use time::format_description::well_known::iso8601::{
    Config, EncodedConfig, Iso8601, TimePrecision,
};

pub fn gmt_format(date_time: &OffsetDateTime) -> String {
    use time::format_description::well_known::Rfc2822;
    date_time.format(&Rfc2822).unwrap().replace("+0000", "GMT")
}

pub fn now_iso8601() -> String {
    const ENCODED_CONFIG: EncodedConfig = Config::DEFAULT
        .set_time_precision(TimePrecision::Second {
            decimal_digits: None,
        })
        .encode();

    OffsetDateTime::now_utc()
        .format(&Iso8601::<ENCODED_CONFIG>)
        .unwrap()
}

pub fn into_header_map(map: HashMap<String, String>) -> HeaderMap {
    map.iter()
        .map(|(k, v)| {
            let name = HeaderName::from_bytes(k.as_bytes()).unwrap();
            let value = HeaderValue::from_bytes(v.as_bytes()).unwrap();
            (name, value)
        })
        .collect()
}

pub fn sign_hmac_sha1(secret: &str, str_to_sign: &str) -> Vec<u8> {
    type HmacSha1 = Hmac<Sha1>;
    let mut mac = HmacSha1::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(str_to_sign.as_bytes());
    mac.finalize().into_bytes().to_vec()
}
