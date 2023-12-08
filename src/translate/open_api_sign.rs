use crate::email::utils::now_iso8601;
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};
use time::OffsetDateTime;
use url::Url;

use crate::error::Error;

// 阿里云签名文档链接：https://help.aliyun.com/zh/sdk/product-overview/v3-request-structure-and-signature

// pub struct CommonRequestHeaders {
//     pub x_acs_action: String,
//     pub x_acs_version: String,
//     pub authorization: String,
//     pub x_acs_signature_nonce: String,
//     pub x_acs_date: String,
//     pub host: String,
//     pub x_acs_content_sha256: String,
//     pub x_acs_security_token: Option<String>,
// }

pub struct SignParams<'a> {
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
// 如果资源路径uri为`None`，则返回`/`
pub(crate) fn generate_can_uri(
    host: &str,
    query_map: &BTreeMap<String, String>,
) -> Result<(String, String, String), Error> {
    let u = Url::parse_with_params(&format!("https://{}", host), query_map)
        .map_err(|_| Error::CommonError("url parsed failed!".to_owned()))?;
    let can_uri = u
        .path()
        .to_owned()
        // Url::parse过后，即percentEncode后，按照文档替换相关字符
        ;
    let can_query_str;
    if let Some(s) = u.query() {
        can_query_str = s
            .replace('+', "%20")
            .replace('*', "%2A")
            .replace("%7E", "~")
    } else {
        can_query_str = "".to_owned()
    };
    Ok((can_uri, u.to_string(), can_query_str))
}

pub struct GenerateCanHeadersRes {
    pub can_headers: String,
    pub can_signed_headers: String,
    pub common_headers: HashMap<String, String>,
}
// CanonicalizedHeaders
// host可选，按照文档没host也行的，但是加个host安全点
// return (can_headers, can_signed_headers)
pub(crate) fn generate_can_headers(
    x_header: Option<&BTreeMap<String, String>>,
    host: Option<&str>,
    x_acs_action: &str,
    x_acs_version: &str,
    x_acs_security_token: Option<&str>,
    x_acs_content_sha256: &str,
) -> GenerateCanHeadersRes {
    let mut need_signed_headers = BTreeMap::new();
    need_signed_headers.insert("x-acs-action".to_owned(), x_acs_action.to_owned());
    need_signed_headers.insert("x-acs-version".to_owned(), x_acs_version.to_owned());
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
    if let Some(h) = host {
        need_signed_headers.insert("host".to_owned(), h.trim().to_owned());
    };
    if let Some(s) = x_acs_security_token {
        need_signed_headers.insert("x-acs-security-token".to_owned(), s.trim().to_owned());
    }
    need_signed_headers.insert(
        "x-acs-content-sha256".to_owned(),
        x_acs_content_sha256.to_owned(),
    );

    let mut can_headers = String::new();
    let mut can_signed_headers = String::new();
    for (k, v) in need_signed_headers {
        can_headers.push_str(format!("{k}:{v}\n").as_str());
        can_signed_headers.push_str(format!("{k};").as_str());
    }
    // 删除最后一个没用的`;`
    can_signed_headers.pop();

    let mut common_headers = HashMap::new();
    common_headers.insert("x-acs-action".to_owned(), x_acs_action.to_owned());
    common_headers.insert("x-acs-version".to_owned(), x_acs_version.to_owned());
    common_headers.insert("Authorization".to_owned(), "".to_owned());
    common_headers.insert("x-acs-signature-nonce".to_owned(), timestamp);
    common_headers.insert("x-acs-date".to_owned(), date);
    common_headers.insert("host".to_owned(), "".to_owned());
    common_headers.insert(
        "x-acs-content-sha256".to_owned(),
        x_acs_content_sha256.to_owned(),
    );
    if let Some(s) = x_acs_security_token {
        common_headers.insert("x-acs-security-token".to_owned(), s.to_owned());
    }

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
) -> (HashMap<String, String>, String) {
    // region    --- sign authorization
    let (can_uri, url_, can_query_str) =
        generate_can_uri(sign_params.host, sign_params.query_map).unwrap();
    // println!("can_uri:{}\nurl:{}", can_uri, url_);
    // 选择使用文档中的ACS3-HMAC-SHA256算法
    let body_hash = hash_sha256(sign_params.body_bytes);
    let generate_can_headers_res = generate_can_headers(
        sign_params.x_headers,
        Some(sign_params.host),
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
    // println!("can_req_str:\n{:#?}", can_req_str);
    let hashed_str = hash_sha256(Some(can_req_str.as_bytes()));
    let str_to_sign = format!("ACS3-HMAC-SHA256\n{hashed_str}");
    // println!("str_to_sign:\n{:#?}", str_to_sign);
    let signature = sign_hmac_sha256(access_key_secret, &str_to_sign);
    // println!("signature:\n{:#?}", signature);
    // endregion --- sign authorization

    let mut common_headers = generate_can_headers_res.common_headers;
    // 覆盖没有的值
    let authorization = format!(
        "ACS3-HMAC-SHA256 Credential={},SignedHeaders={},Signature={}",
        access_key_id, generate_can_headers_res.can_signed_headers, signature
    );
    common_headers.insert("Authorization".to_owned(), authorization);
    common_headers.insert("host".to_owned(), sign_params.host.to_owned());

    (common_headers, url_)
}
