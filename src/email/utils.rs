use base64::{engine::general_purpose, Engine};
use hmac::{Hmac, Mac};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use sha1::Sha1;
use std::collections::BTreeMap;
use time::format_description::well_known::iso8601::{
    Config, EncodedConfig, Iso8601, TimePrecision,
};
use url::form_urlencoded;

// 定义一个函数，用于计算签名字符串
pub fn sign_params(query_params: &BTreeMap<String, String>, access_key_secret: &str) -> String {
    let canonicalized_query_string = form_urlencoded::Serializer::new(String::new())
        .extend_pairs(query_params)
        .finish()
        .replace("+", "%20")
        .replace("*", "%2A")
        .replace("%7E", "~");

    let percent_encode_string = utf8_percent_encode(&canonicalized_query_string, NON_ALPHANUMERIC)
        .to_string()
        .replace("%2D", "-")
        .replace("%5F", "_")
        .replace("%2E", ".");

    let string_to_sign = format!("{}&{}&{}", "POST", "%2F", percent_encode_string);

    type HmacSha1 = Hmac<Sha1>;
    let mut mac =
        HmacSha1::new_from_slice(format!("{}&", access_key_secret).as_bytes()).expect("error key");
    mac.update(string_to_sign.as_bytes());
    let res = mac.finalize().into_bytes();

    general_purpose::STANDARD.encode(res)
}

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

pub fn get_uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}

#[test]
fn sign_params_test() {
    let s = "AccessKeyId=testid&AccountName=<a%b'>&Action=SingleSendMail&AddressType=1&Format=XML&HtmlBody=4&RegionId=cn-hangzhou&ReplyToAddress=true&SignatureMethod=HMAC-SHA1&SignatureNonce=c1b2c332-4cfb-4a0f-b8cc-ebe622aa0a5c&SignatureVersion=1.0&Subject=3&TagName=2&Timestamp=2016-10-20T06:27:56Z&ToAddress=1@test.com&Version=2015-11-23";
    let url = url::Url::parse(&format!("http://example.com?{}", s)).unwrap();
    let mut map = BTreeMap::new();
    for (key, value) in url.query_pairs() {
        map.insert(key.to_string(), value.to_string());
    }

    let sign = sign_params(&map, "testsecret");

    assert_eq!(sign, "llJfXJjBW3OacrVgxxsITgYaYm0=")
}
