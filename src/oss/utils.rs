use crate::error::Error;
use crate::utils::common::sign_hmac_sha1;
use base64::{engine::general_purpose, Engine};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::collections::{BTreeMap, HashMap};

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
            return Err(Error::AnyError(
                "unknown type(get canonicalized)".to_owned(),
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
    let content_md5 = content_md5.unwrap_or_default();

    let canonicalized_oss_headers = if oss_header_map.is_some() {
        get_canonicalized_oss_header(oss_header_map)
    } else {
        "".to_owned()
    };
    let canonicalized_resource = get_canonicalized_resource(bucket_name, object_name).unwrap();

    let content_type = content_type.unwrap_or_default();

    let str_to_sign = format!(
        "{}\n{}\n{}\n{}\n{}{}",
        verb, content_md5, content_type, date, canonicalized_oss_headers, canonicalized_resource
    );
    // println!("str_to_sign:{}", str_to_sign);

    let res = sign_hmac_sha1(access_key_secret, &str_to_sign);
    let signature = general_purpose::STANDARD.encode(res);

    format!("OSS {}:{}", access_key_id, signature)
}

#[test]
fn get_content_md5_test() {
    let s = get_content_md5(b"0123456789");
    assert_eq!(&s, "eB5eJF1ptWaXm4bijSPyxw==")
}
pub(crate) fn into_request_header(map: HashMap<&str, &str>) -> HeaderMap {
    map.into_iter()
        .map(|(k, v)| {
            let name = HeaderName::from_bytes(k.as_bytes()).unwrap();
            let value = HeaderValue::from_bytes(v.as_bytes()).unwrap();
            (name, value)
        })
        .collect()
}

pub(crate) async fn handle_response_status(resp: reqwest::Response) -> Result<String, Error> {
    let status = resp.status();
    let text = resp.text().await?;
    if !status.is_success() {
        return Err(Error::RequestAPIFailed {
            status: status.to_string(),
            text,
        });
    }
    Ok(text)
}
