use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use url::{form_urlencoded, Url};

use crate::error::Error;

// 阿里云签名文档链接：https://help.aliyun.com/zh/sdk/product-overview/v3-request-structure-and-signature

// CanonicalURI
// 如果资源路径uri为`None`，则返回`/`
pub(crate) fn generate_can_uri(url_: &str) -> Result<String, Error> {
    let u = Url::parse(url_).map_err(Error::CommonError("url parsed failed!".to_owned()))?;
    let res = u
        .path()
        .to_owned()
        // Url::parse过后，即percentEncode后，按照文档替换相关字符
        .replace("%2D", "-")
        .replace("%5F", "_")
        .replace("%2E", ".");
    Ok(res)
}

// CanonicalQueryString
pub(crate) fn generate_can_query_str(query_map: &BTreeMap<String, String>) -> String {
    form_urlencoded::Serializer::new(String::new())
        .extend_pairs(query_map)
        .finish()
}

// CanonicalizedHeaders
// host可选，按照文档没host也行的，但是加个host安全点
// return (can_headers, can_signed_headers)
pub(crate) fn generate_can_headers(
    x_header: &BTreeMap<String, String>,
    host: Option<&str>,
) -> (String, String) {
    let mut x_header: BTreeMap<String, String> = x_header
        .into_iter()
        .map(|(k, v)| (k.to_lowercase(), v.trim().to_string()))
        .collect();
    if let Some(h) = host {
        x_header.insert("host".to_owned(), h.trim().to_owned());
    };
    let mut can_headers = String::new();
    let mut can_signed_headers = String::new();
    for (k, v) in &x_header {
        can_headers.push_str(format!("{k}:{v}\n").as_str());
        can_signed_headers.push_str(format!("{k};").as_str());
    }
    // 删除最后一个没用的`;`
    can_signed_headers.pop();

    (can_headers, can_signed_headers)
}

pub(crate) fn hash_sha256(bytes: Option<&[u8]>) -> String {
    if let Some(b) = bytes {
        let mut hasher = Sha256::new();
        hasher.update(b);
        let res = hasher.finalize();
        hex::encode(res)
    } else {
        "".to_owned()
    }
}

pub(crate) fn sign_hmac_sha256(secret: &str, str_to_sign: &str) -> String {
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(str_to_sign.as_bytes());
    let res = mac.finalize().into_bytes();
    hex::encode(res)
}

// Authorization
pub(crate) fn sign_authorization(
    access_key_secret: &str,
    req_method: &str,
    host: &str,
    url: &str,
    query_map: &BTreeMap<String, String>,
    x_headers: &BTreeMap<String, String>,
    body_bytes: Some(&[u8]),
) -> String {
    let can_uri = generate_can_uri(url).unwrap();
    let can_query_str = generate_can_query_str(query_map);
    let (can_headers, can_signed_headers) = generate_can_headers(x_headers, Some(host));
    // 选择使用文档中的ACS3-HMAC-SHA256算法
    let body_hash = hash_sha256(body_bytes);

    let can_req_str = format!("{req_method}\n{can_uri}\n{can_query_str}\n{can_headers}\n{can_signed_headers}\n{body_hash}");
    let hashed_str = hash_sha256(Some(can_req_str.as_bytes()));
    let str_to_sign = format!("ACS3-HMAC-SHA256\n{hashed_str}");
    sign_hmac_sha256(access_key_secret, &str_to_sign)
}
