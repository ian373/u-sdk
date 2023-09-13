use base64::{engine::general_purpose, Engine};
use hmac::{Hmac, Mac};
use sha1::Sha1;
use std::collections::BTreeMap;

use crate::error::Error;

pub fn now_gmt() -> String {
    use time::format_description::well_known::Rfc2822;
    time::OffsetDateTime::now_utc()
        .format(&Rfc2822)
        .unwrap()
        .replace("+0000", "GMT")
}

// TODO 注意，这个方法理论上是私有的，文件外部只有一个地方使用，到时候当外部不使用的时候记得改为私有
pub fn get_content_md5(bytes: Option<&[u8]>) -> String {
    if bytes.is_none() {
        return "".to_owned();
    }

    use md5::{Digest, Md5};

    let mut hasher = Md5::new();
    hasher.update(bytes.unwrap());
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
    bytes: Option<&[u8]>,
    content_type: Option<&str>,
    date: &str,
    oss_header_map: Option<&BTreeMap<String, String>>,
    bucket_name: Option<&str>,
    object_name: Option<&str>,
) -> String {
    let content_md5 = if bytes.is_some() {
        get_content_md5(bytes)
    } else {
        "".to_owned()
    };

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
    let s = get_content_md5(Some(b"0123456789"));
    assert_eq!(&s, "eB5eJF1ptWaXm4bijSPyxw==")
}
