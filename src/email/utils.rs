use base64::engine::{general_purpose, Engine};
use percent_encoding::{percent_encode, AsciiSet, NON_ALPHANUMERIC};
use std::collections::BTreeMap;
use url::form_urlencoded;

use crate::utils::common::sign_hmac_sha1;

// 签名文档：https://help.aliyun.com/document_detail/29442.html

// 定义一个函数，用于计算签名字符串
pub fn sign_params(query_params: &BTreeMap<String, String>, access_key_secret: &str) -> String {
    let canonicalized_query_string = form_urlencoded::Serializer::new(String::new())
        .extend_pairs(query_params)
        .finish()
        .replace('+', "%20")
        .replace('*', "%2A")
        .replace("%7E", "~");

    // 下边四个字符不用编码，移出需要编码的字符集
    const FRAGMENT: &AsciiSet = &NON_ALPHANUMERIC
        .remove(b'-')
        .remove(b'_')
        .remove(b'.')
        .remove(b'~');
    let percent_encode_string =
        percent_encode(canonicalized_query_string.as_bytes(), FRAGMENT).to_string();

    let string_to_sign = format!("{}&{}&{}", "POST", "%2F", percent_encode_string);

    let secret = format!("{}&", access_key_secret);
    let res = sign_hmac_sha1(&secret, &string_to_sign);
    general_purpose::STANDARD.encode(res)
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
