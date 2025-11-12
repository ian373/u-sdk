use crate::oss::Client;
use crate::oss::Error;
use crate::oss::sign_v4::{HTTPVerb, SignV4Param};
use base64::{Engine, engine::general_purpose};
use md5::{Digest, Md5};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;
use tokio::io::AsyncReadExt;
use u_sdk_common::helper::gmt_format;
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

pub(crate) async fn into_request_failed_error(resp: reqwest::Response) -> Error {
    let status = resp.status();
    let body = resp.text().await;
    match body {
        Ok(text) => Error::RequestAPIFailed {
            status: status.to_string(),
            text,
        },
        Err(e) => Error::Reqwest(e),
    }
}

// TODO 放到common-lib中供全局使用
pub(crate) async fn parse_xml_response<T: serde::de::DeserializeOwned>(
    resp: reqwest::Response,
) -> Result<T, Error> {
    let status = resp.status();

    if !status.is_success() {
        return Err(into_request_failed_error(resp).await);
    }

    let text = resp.text().await?;
    let data = quick_xml::de::from_str(&text)
        .map_err(|e| Error::Common(format!("XML parse error: {}", e)))?;
    Ok(data)
}

// 用 buffer 读文件并计算MD5
pub(crate) async fn compute_md5_from_file(path: &Path) -> Result<String, Error> {
    let mut file = tokio::fs::File::open(path).await?;
    let mut hasher = Md5::new();
    // 放到堆上并初始化
    let mut buf = vec![0u8; 64 * 1024]; // 64KB buffer

    loop {
        let n = file.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }

    let result = hasher.finalize();
    Ok(general_purpose::STANDARD.encode(result))
}

pub(crate) fn validate_object_name(name: &str) -> Result<(), Error> {
    // 1. 长度检查
    let len = name.len();
    if len == 0 {
        return Err(Error::Common("object_name cannot be empty".to_owned()));
    }
    if len > 1023 {
        return Err(Error::Common(
            "object_name is too long, max is 1023 bytes".to_owned(),
        ));
    }

    // 2. 前缀检查
    if let Some(first) = name.chars().next()
        && (first == '/' || first == '\\')
    {
        return Err(Error::Common(
            "object_name cannot start with '/' or '\\'".to_owned(),
        ));
    }

    // 3. 控制字符检查
    if name.bytes().any(|b| b == b'\r' || b == b'\n') {
        return Err(Error::Common(
            "object_name cannot contain control characters".to_owned(),
        ));
    }

    // 4. 空路径段检查：先 split，再忽略末尾因 '/' 产生的空段，再拒绝中间任何空字符串
    let segments: Vec<&str> = name.split('/').collect();
    let to_check = if name.ends_with('/') {
        &segments[..segments.len().saturating_sub(1)]
    } else {
        &segments[..]
    };
    if to_check.iter().any(|seg| seg.is_empty()) {
        return Err(Error::Common(
            "object_name cannot contain empty path segments".to_owned(),
        ));
    }

    // 5. 相对路径段检查
    for segment in to_check {
        if *segment == "." || *segment == ".." {
            return Err(Error::Common(
                "object_name cannot contain relative path segments '.' or '..'".to_owned(),
            ));
        }
    }

    Ok(())
}

#[test]
fn validate_object_name_test() {
    // 合法示例：长度、前缀、末尾斜杠、普通子目录、UTF-8 字符
    assert!(validate_object_name("exampleobject.txt").is_ok());
    assert!(validate_object_name("dir/subdir/file_测试-01.log").is_ok());
    assert!(validate_object_name("a/b").is_ok());
    assert!(validate_object_name("a/b/").is_ok());

    // 非法示例：长度为 0、超过 1023 字节
    assert!(validate_object_name("").is_err());
    assert!(validate_object_name(&"a".repeat(1024)).is_err());

    // 非法示例：前缀以 '/' 或 '\' 开头
    assert!(validate_object_name("/badname").is_err());
    assert!(validate_object_name("\\badname").is_err());

    // 非法示例：包含控制字符
    assert!(validate_object_name("bad\r\nname").is_err());

    // 非法示例：连续斜杠导致的空路径段
    assert!(validate_object_name("a//abc").is_err());
    assert!(validate_object_name("x/y//z").is_err());

    // 非法示例：相对路径段 '.' 或 '..'
    assert!(validate_object_name("./abc").is_err());
    assert!(validate_object_name("../abc").is_err());
    assert!(validate_object_name("abc/./def").is_err());
    assert!(validate_object_name("abc/../def").is_err());
}

// 少数api需要指定和client不同的region和bucket，使用这个方法进行签名计算，同时也作为签名代码的实现
pub(crate) fn get_request_header_with_bucket_region(
    client: &Client,
    req_header_map: HashMap<String, String>,
    request_url: &Url,
    http_verb: HTTPVerb,
    signing_region: &str,
    bucket: Option<&str>,
) -> HeaderMap {
    // 把需要签名的header和不需要签名的header分开
    let (sign_map, remaining_map) = partition_header(req_header_map);

    // 创建CanonicalHeaders，把所有需要签名的header放到CanonicalHeader中
    let mut canonical_header = BTreeMap::new();
    canonical_header.extend(sign_map.iter().map(|(k, v)| (k.as_str(), v.as_str())));

    // x-oss-content-sha256是必须存在且值固定
    canonical_header.insert("x-oss-content-sha256", "UNSIGNED-PAYLOAD");
    // host为addition_header中指定的需要额外添加到签名计算中的参数
    canonical_header.insert("host", request_url.host_str().unwrap());

    // 添加host到additional_header，因为canonical_header中把host也参与签名计算了
    let mut additional_header = BTreeSet::new();
    additional_header.insert("host");
    let now = time::OffsetDateTime::now_utc();
    let sign_v4_param = SignV4Param {
        signing_region,
        http_verb,
        uri: request_url,
        bucket,
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

// 大部分api的签名都是默认使用client的region和bucket，使用这个方法
pub(crate) fn get_request_header(
    client: &Client,
    req_header_map: HashMap<String, String>,
    request_url: &Url,
    http_verb: HTTPVerb,
) -> HeaderMap {
    get_request_header_with_bucket_region(
        client,
        req_header_map,
        request_url,
        http_verb,
        &client.region,
        Some(&client.bucket),
    )
}

pub(crate) fn get_date_str(data: &time::OffsetDateTime) -> String {
    let date_format = time::format_description::parse("[year][month][day]").unwrap();
    data.format(&date_format).unwrap()
}

pub(crate) fn get_date_time_str(data: &time::OffsetDateTime) -> String {
    let data_time_format =
        time::format_description::parse("[year][month][day]T[hour][minute][second]Z").unwrap();
    data.format(&data_time_format).unwrap()
}

pub(crate) fn generate_presigned_url(
    client: &Client,
    header_map: HashMap<String, String>,
    mut presigned_url: Url,
    http_verb: HTTPVerb,
    url_expires: i32,
) -> String {
    let (header_map, remaining_map) = partition_header(header_map);
    let mut canonical_header = BTreeMap::new();
    canonical_header.extend(header_map.iter().map(|(k, v)| (k.as_str(), v.as_str())));
    let host = presigned_url.host_str().unwrap().to_owned();
    canonical_header.insert("host", host.as_str());

    let mut additional_header = BTreeSet::new();
    additional_header.insert("host");

    for (k, v) in &remaining_map {
        canonical_header.insert(k.as_str(), v.as_str());
        additional_header.insert(k.as_str());
    }

    let now = time::OffsetDateTime::now_utc();
    // 添加参与签名计算的query
    presigned_url
        .query_pairs_mut()
        .append_pair("x-oss-signature-version", "OSS4-HMAC-SHA256")
        .append_pair(
            "x-oss-credential",
            &format!(
                "{}/{}/{}/oss/aliyun_v4_request",
                client.access_key_id,
                &get_date_str(&now),
                client.region
            ),
        )
        .append_pair("x-oss-date", &get_date_time_str(&now))
        .append_pair("x-oss-expires", &url_expires.to_string())
        .append_pair(
            "x-oss-additional-headers",
            additional_header
                .iter()
                .cloned()
                .collect::<Vec<&str>>()
                .join(";")
                .as_str(),
        )
        .finish();

    let sign_v4_param = SignV4Param {
        signing_region: &client.region,
        http_verb,
        uri: &presigned_url,
        bucket: Some(&client.bucket),
        header_map: &canonical_header,
        additional_header: Some(&additional_header),
        date_time: &now,
    };
    let signature = client.generate_v4_signature(sign_v4_param);
    // 补上不参与签名计算的query
    presigned_url
        .query_pairs_mut()
        .append_pair("x-oss-signature", &signature)
        .finish();

    presigned_url.to_string()
}

// 将Header分为需要参与签名的Header和剩余Header
fn partition_header(
    header_map: HashMap<String, String>,
) -> (HashMap<String, String>, HashMap<String, String>) {
    let mut sign_map = HashMap::new();
    let mut remaining_map = HashMap::new();
    for (k, v) in header_map {
        let k = k.to_lowercase();
        if k == "content-type" || k == "content-md5" || k.starts_with("x-oss-") {
            sign_map.insert(k, v);
        } else {
            remaining_map.insert(k, v);
        }
    }
    (sign_map, remaining_map)
}

pub(crate) fn parse_get_object_response_header<T: DeserializeOwned>(
    header: &HeaderMap,
) -> (T, HashMap<String, String>) {
    let mut map = Map::with_capacity(30);
    let mut custom_meta_map = HashMap::with_capacity(30);
    for (name, val) in header {
        let name_s = name.as_str();
        if let Ok(s) = val.to_str() {
            if name_s.starts_with("x-oss-meta-") {
                custom_meta_map.insert(
                    name_s.trim_start_matches("x-oss-meta-").to_string(),
                    s.to_string(),
                );
            } else {
                map.insert(name_s.to_string(), Value::String(s.to_string()));
            }
        }
    }

    let response_header = serde_json::from_value::<T>(Value::Object(map)).unwrap();
    (response_header, custom_meta_map)
}
