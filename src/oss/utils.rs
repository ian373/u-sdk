use base64::{engine::general_purpose, Engine};
use hmac::{Hmac, Mac};
use sha1::Sha1;
use std::collections::BTreeMap;

use crate::error::Error;

pub fn get_content_md5(bytes: &[u8]) -> String {
    use md5::{Digest, Md5};

    let mut hasher = Md5::new();
    hasher.update(bytes);
    let res = hasher.finalize();

    general_purpose::STANDARD.encode(res)
}

fn get_canonicalized_oss_header(oss_header_map: Option<&BTreeMap<String, String>>) -> String {
    if oss_header_map.is_none() || oss_header_map.unwrap().is_empty() {
        return "".to_owned();
    }

    let mut s = String::new();

    for (k, v) in oss_header_map.unwrap() {
        let pair = format!("{}:{}\n", k.to_lowercase(), v);
        s.push_str(&pair);
    }

    s
}

fn get_canonicalized_resource(
    bucket_name: Option<&str>,
    object_name: Option<&str>,
) -> Result<String, Error> {
    let s = match (bucket_name, object_name) {
        (Some(b), Some(o)) => format!("/{b}/{o}"),
        (Some(b), None) => format!("/{b}/"),
        (None, None) => "/".to_owned(),
        _ => {
            return Err(Error::CommonError(
                "unknow type(get canonicalized)".to_owned(),
            ));
        }
    };
    Ok(s)
}

#[allow(clippy::too_many_arguments)]
pub fn sign_authorization(
    access_key_id: &str,
    access_key_secret: &str,
    verb: &str,
    content_md5: Option<&str>,
    content_type: Option<&str>,
    date: &str,
    oss_header_map: Option<&BTreeMap<String, String>>,
    bucket_name: Option<&str>,
    object_name: Option<&str>,
) -> String {
    let content_md5 = if let Some(s) = content_md5 { s } else { "" };

    let canonicalized_oss_headers = if oss_header_map.is_some() {
        get_canonicalized_oss_header(oss_header_map)
    } else {
        "".to_owned()
    };
    let canonicalized_resource = get_canonicalized_resource(bucket_name, object_name).unwrap();

    let content_type = if let Some(s) = content_type { s } else { "" };

    let str_to_sign = format!(
        "{}\n{}\n{}\n{}\n{}{}",
        verb, content_md5, content_type, date, canonicalized_oss_headers, canonicalized_resource
    );
    // println!("str_to_sign:{}", str_to_sign);

    type HmacSha1 = Hmac<Sha1>;
    let mut mac = HmacSha1::new_from_slice(access_key_secret.as_bytes()).unwrap();
    mac.update(str_to_sign.as_bytes());
    let res = mac.finalize().into_bytes();
    let signature = general_purpose::STANDARD.encode(res);

    format!("OSS {}:{}", access_key_id, signature)
}

#[test]
fn get_content_md5_test() {
    let s = get_content_md5(b"0123456789");
    assert_eq!(&s, "eB5eJF1ptWaXm4bijSPyxw==")
}

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::collections::HashMap;
pub fn into_header_map(map: HashMap<String, String>) -> HeaderMap {
    map.iter()
        .map(|(k, v)| {
            let name = HeaderName::from_bytes(k.as_bytes()).unwrap();
            let value = HeaderValue::from_bytes(v.as_bytes()).unwrap();
            (name, value)
        })
        .collect()
}
