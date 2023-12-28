use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};
use time::OffsetDateTime;
use url::Url;

use super::common::now_iso8601;
use crate::error::Error;

// 阿里云签名文档链接：https://help.aliyun.com/zh/sdk/product-overview/v3-request-structure-and-signature

pub(crate) struct SignParams<'a> {
    pub req_method: &'a str,
    pub host: &'a str,
    pub query_map: &'a BTreeMap<String, String>,
    pub x_headers: Option<&'a BTreeMap<String, String>>,
    pub body_bytes: Option<&'a [u8]>,
    pub x_acs_action: &'a str,
    pub x_acs_version: &'a str,
    pub x_acs_security_token: Option<&'a str>,
}

// CanonicalURI
// return: (CanonicalURI, 完整的url用于发送http请求, CanonicalQueryString)
pub(crate) fn generate_can_uri(
    host: &str,
    query_map: &BTreeMap<String, String>,
) -> Result<(String, String, String), Error> {
    let u = Url::parse_with_params(&format!("https://{}", host), query_map)
        .map_err(|_| Error::CommonError("url parsed failed!".to_owned()))?;
    let can_uri = u.path().to_owned();
    let can_query_str;
    if let Some(s) = u.query() {
        // Url::parse过后，即percentEncode后，按照文档替换相关字符
        can_query_str = s
            .replace('+', "%20")
            .replace('*', "%2A")
            .replace("%7E", "~")
    } else {
        can_query_str = "".to_owned()
    };
    Ok((can_uri, u.to_string(), can_query_str))
}

pub(crate) struct GenerateCanHeadersRes {
    // CanonicalHeaders
    pub can_headers: String,
    // SignedHeaders
    pub can_signed_headers: String,
    // 公共请求头，当api调用的时候，直接把公共请求头加入请求的headers即可
    pub common_headers: HashMap<String, String>,
}
// CanonicalizedHeaders
pub(crate) fn generate_can_headers(
    x_header: Option<&BTreeMap<String, String>>,
    // 按照官方签名文档没提到要host，但是官方签名示例加了个host，所以这里也加个host
    host: &str,
    x_acs_action: &str,
    x_acs_version: &str,
    x_acs_security_token: Option<&str>,
    x_acs_content_sha256: &str,
) -> GenerateCanHeadersRes {
    let mut need_signed_headers = BTreeMap::new();
    need_signed_headers.insert("x-acs-action".to_owned(), x_acs_action.to_owned());
    need_signed_headers.insert("x-acs-version".to_owned(), x_acs_version.to_owned());
    // 使用时间戳当随机数
    let timestamp = OffsetDateTime::now_utc().unix_timestamp_nanos().to_string();
    need_signed_headers.insert("x-acs-signature-nonce".to_owned(), timestamp.clone());
    let date = now_iso8601();
    need_signed_headers.insert("x-acs-date".to_owned(), date.clone());

    if let Some(h) = x_header {
        let x_header: BTreeMap<String, String> = h
            .iter()
            .map(|(k, v)| (k.to_lowercase(), v.trim().to_owned()))
            .collect();
        need_signed_headers.extend(x_header);
    }
    need_signed_headers.insert("host".to_owned(), host.trim().to_owned());
    if let Some(s) = x_acs_security_token {
        need_signed_headers.insert("x-acs-security-token".to_owned(), s.trim().to_owned());
    }
    need_signed_headers.insert(
        "x-acs-content-sha256".to_owned(),
        x_acs_content_sha256.to_owned(),
    );

    let mut can_headers = String::new();
    let mut can_signed_headers = String::new();
    for (k, v) in &need_signed_headers {
        can_headers.push_str(format!("{k}:{v}\n").as_str());
        can_signed_headers.push_str(format!("{k};").as_str());
    }
    // 删除最后一个没用的`;`
    can_signed_headers.pop();

    let common_headers = need_signed_headers.into_iter().collect::<HashMap<_, _>>();

    GenerateCanHeadersRes {
        can_headers,
        can_signed_headers,
        common_headers,
    }
}

pub(crate) fn hash_sha256(bytes: Option<&[u8]>) -> String {
    let mut hasher = Sha256::new();
    if let Some(b) = bytes {
        hasher.update(b);
    } else {
        hasher.update(b"");
    };
    let hash_str = hasher.finalize();
    hex::encode(hash_str)
}

pub(crate) fn sign_hmac_sha256(secret: &str, str_to_sign: &str) -> String {
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(str_to_sign.as_bytes());
    let res = mac.finalize().into_bytes();
    hex::encode(res)
}

pub(crate) fn get_common_headers(
    access_key_secret: &str,
    access_key_id: &str,
    sign_params: SignParams,
) -> Result<(HashMap<String, String>, String), Error> {
    // region    --- sign authorization
    let (can_uri, url_, can_query_str) = generate_can_uri(sign_params.host, sign_params.query_map)?;
    // 选择使用文档中的ACS3-HMAC-SHA256算法
    let body_hash = hash_sha256(sign_params.body_bytes);
    let generate_can_headers_res = generate_can_headers(
        sign_params.x_headers,
        sign_params.host,
        sign_params.x_acs_action,
        sign_params.x_acs_version,
        sign_params.x_acs_security_token,
        &body_hash,
    );

    let can_req_str = format!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        sign_params.req_method,
        can_uri,
        can_query_str,
        generate_can_headers_res.can_headers,
        generate_can_headers_res.can_signed_headers,
        body_hash
    );
    let hashed_str = hash_sha256(Some(can_req_str.as_bytes()));
    let str_to_sign = format!("ACS3-HMAC-SHA256\n{hashed_str}");
    let signature = sign_hmac_sha256(access_key_secret, &str_to_sign);
    // endregion --- sign authorization

    let mut common_headers = generate_can_headers_res.common_headers;
    let authorization = format!(
        "ACS3-HMAC-SHA256 Credential={},SignedHeaders={},Signature={}",
        access_key_id, generate_can_headers_res.can_signed_headers, signature
    );
    // 把Authorization加入common_headers构成最终的公共请求头
    common_headers.insert("Authorization".to_owned(), authorization);

    Ok((common_headers, url_))
}
