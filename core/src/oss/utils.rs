use crate::error::Error;
use base64::{Engine, engine::general_purpose};
use md5::{Digest, Md5};
use regex::Regex;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;
use tokio::io::AsyncReadExt;

pub fn get_content_md5(bytes: &[u8]) -> String {
    use md5::{Digest, Md5};

    let mut hasher = Md5::new();
    hasher.update(bytes);
    let res = hasher.finalize();

    general_purpose::STANDARD.encode(res)
}

#[test]
fn get_content_md5_test() {
    let s = get_content_md5(b"0123456789");
    assert_eq!(&s, "eB5eJF1ptWaXm4bijSPyxw==")
}

// TODO 这个到时后要和`common-lib/src/helper.rs`的合并
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

// TODO 这个trait没必要存在，到时后删了
pub(crate) trait SerializeToHashMap
where
    Self: Sized + Serialize,
{
    fn serialize_to_hashmap(&self) -> Result<HashMap<String, String>, Error> {
        let r = serde_json::from_value(serde_json::to_value(self)?)?;
        Ok(r)
    }
}

// 用 16KB 缓冲流式读文件并计算MD5
pub(crate) async fn compute_md5_from_file(path: &Path) -> Result<String, Error> {
    let mut file = tokio::fs::File::open(path)
        .await
        .map_err(|e| Error::AnyError(format!("open file error: {}", e)))?;
    let mut hasher = Md5::new();
    let mut buf = [0u8; 16 * 1024]; // 16 KB

    loop {
        let n = file
            .read(&mut buf)
            .await
            .map_err(|e| Error::AnyError(format!("read file error: {}", e)))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }

    let result = hasher.finalize();
    Ok(general_purpose::STANDARD.encode(result))
}

pub(crate) fn is_valid_object_name(name: &str) -> bool {
    let re = Regex::new(r"^/[^/]+(?:/[^/]+)*/?$").unwrap();
    re.is_match(name)
}

#[test]
fn validate_object_name_test() {
    assert!(is_valid_object_name("/foo"));
    assert!(is_valid_object_name("/foo/"));
    assert!(is_valid_object_name("/foo/bar"));
    assert!(is_valid_object_name("/foo/bar/"));
    assert!(!is_valid_object_name("foo"));
    assert!(!is_valid_object_name("/"));
    assert!(!is_valid_object_name("//foo"));
    assert!(!is_valid_object_name("/foo//bar"));
}
