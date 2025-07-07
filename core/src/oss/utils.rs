use crate::error::Error;
use crate::oss::Client;
use crate::oss::object::utils::partition_header;
use crate::oss::sign_v4::{HTTPVerb, SignV4Param};
use base64::{Engine, engine::general_purpose};
use common_lib::helper::gmt_format;
use md5::{Digest, Md5};
use regex::Regex;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;
use tokio::io::AsyncReadExt;
use url::Url;

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

// 用 buffer 读文件并计算MD5
pub(crate) async fn compute_md5_from_file(path: &Path) -> Result<String, Error> {
    let mut file = tokio::fs::File::open(path)
        .await
        .map_err(|e| Error::AnyError(format!("open file error: {}", e)))?;
    let mut hasher = Md5::new();
    // 放到堆上并初始化
    let mut buf = vec![0u8; 64 * 1024]; // 64KB buffer

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

pub(crate) fn get_request_header(
    client: &Client,
    req_header_map: HashMap<String, String>,
    request_url: &Url,
) -> HeaderMap {
    // 把需要签名的header和不需要签名的header分开
    let (sign_map, remaining_map) = partition_header(req_header_map);

    // 创建CanonicalHeaders，把所有需要签名的header放到CanonicalHeader中
    let mut canonical_header = BTreeMap::new();
    canonical_header.extend(sign_map.iter().map(|(k, v)| (k.as_str(), v.as_str())));

    canonical_header.insert("x-oss-content-sha256", "UNSIGNED-PAYLOAD");
    canonical_header.insert("host", request_url.host_str().unwrap());

    let mut additional_header = BTreeSet::new();
    additional_header.insert("host");
    let now = time::OffsetDateTime::now_utc();
    let sign_v4_param = SignV4Param {
        signing_region: &client.region,
        http_verb: HTTPVerb::Put,
        uri: request_url,
        bucket: Some(&client.bucket),
        header_map: &canonical_header,
        additional_header: Some(&additional_header),
        date_time: &now,
    };
    let authorization = client.sign_v4(sign_v4_param);

    // 把canonical_header转化为最终的header，补齐剩下的未参与签名计算的header
    // 包括：剩下必要的公共请求头，api header中的非签名字段
    let mut header = canonical_header.into_iter().collect::<HashMap<_, _>>();
    header.insert("Authorization", &authorization);
    let gmt = gmt_format(&now);
    header.insert("Date", &gmt);
    header.extend(remaining_map.iter().map(|(k, v)| (k.as_str(), v.as_str())));
    into_request_header(header)
}
