use super::error::Error;
use super::helper::now_iso8601;
use hmac::{Hmac, Mac};
use rand::Rng;
use rand::distr::Alphanumeric;
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};
use url::{Url, form_urlencoded};

// 阿里云签名文档链接：https://help.aliyun.com/zh/sdk/product-overview/v3-request-structure-and-signature

pub struct SignParams<'a, T: Serialize> {
    // 签名文档中的RequestHeader
    pub host: &'a str,
    pub x_acs_action: &'a str,
    // x-acs-content-sha256: 自动添加
    // x-acs-date: 自动添加
    // x-acs-signature-nonce: 自动添加
    pub x_acs_version: &'a str,
    // Authorization: 自动添加
    pub x_acs_security_token: Option<&'a str>,

    // 其它签名的时候需要的信息
    pub style: &'a OpenApiStyle,
    pub req_method: &'a str,
    // 这个字段只要是序列化为Value后是Object即可
    pub query_map: T,
    pub request_body: Option<&'a RequestBody<'a>>,
}

fn replace_percent_encode(s: &str) -> String {
    s.replace('+', "%20")
        .replace('*', "%2A")
        .replace("%7E", "~")
}

pub enum OpenApiStyle {
    RPC,
    ROA,
}

/// CanonicalURI
///
/// return: (CanonicalURI, 完整的url用于发送http请求, CanonicalQueryString)
pub fn generate_can_uri(
    host: &str,
    query: &impl Serialize,
    style: &OpenApiStyle,
) -> Result<(String, String, String), Error> {
    let query_map = to_query_map(query);
    let u = Url::parse_with_params(&format!("https://{}", host), query_map)
        .map_err(|_| Error::Common("url parsed failed!".to_owned()))?;
    // 使用的url::Url在构建的时候已经按照规范percentEncode过了，但是和文档提的java.net.URLEncoder一样也需要替换
    let can_uri = match style {
        OpenApiStyle::RPC => "/".to_owned(),
        OpenApiStyle::ROA => replace_percent_encode(u.path()),
    };
    // 如果API的请求参数信息包含了"in":"query"时，需要按照文档进行构造，这里的解决方法是，如果有query就进行处理
    // 传入的query_map已经是排序好的BTreeMap，所以直接使用url::Url构造的query部分就是排序好的
    // 这里query需要按照文档进行替换，从url::Url获取的
    // NOTE 签名文档提到`当请求参数类型是array、object时，需要将参数值转化为带索引的键值对`，但是示例代码自己都是(&str, &str)，所以先不处理这种情况
    let can_query_str = if let Some(s) = u.query() {
        replace_percent_encode(s)
    } else {
        "".to_owned()
    };
    Ok((can_uri, u.to_string(), can_query_str))
}

pub struct GenerateCanHeadersRes {
    // CanonicalHeaders
    pub can_headers: String,
    // SignedHeaders
    pub can_signed_headers: String,
    // 公共请求头，当api调用的时候，直接把公共请求头加入请求的headers即可
    pub common_headers: HashMap<String, String>,
}

/// 传入后获得的签名，在发送的时候必须要和传入的一致
pub enum RequestBody<'a> {
    FormData(&'a [(&'a str, &'a str)]),
    Json(&'a str),
    Binary(&'a [u8]),
}

// CanonicalizedHeaders
pub fn generate_can_headers(
    host: &str,
    x_acs_action: &str,
    x_acs_version: &str,
    x_acs_security_token: Option<&str>,
    x_acs_content_sha256: &str,
    content_type: Option<&str>,
) -> GenerateCanHeadersRes {
    let mut need_signed_headers = BTreeMap::new();
    need_signed_headers.insert("host".to_owned(), host.trim().to_owned());
    need_signed_headers.insert("x-acs-action".to_owned(), x_acs_action.to_owned());
    need_signed_headers.insert(
        "x-acs-content-sha256".to_owned(),
        x_acs_content_sha256.to_owned(),
    );
    let date = now_iso8601();
    need_signed_headers.insert("x-acs-date".to_owned(), date.clone());
    need_signed_headers.insert("x-acs-signature-nonce".to_owned(), generate_nonce());
    need_signed_headers.insert("x-acs-version".to_owned(), x_acs_version.to_owned());
    if let Some(s) = x_acs_security_token {
        need_signed_headers.insert("x-acs-security-token".to_owned(), s.trim().to_owned());
    }
    if let Some(ct) = content_type {
        need_signed_headers.insert("content-type".to_owned(), ct.to_owned());
    }

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

pub fn hash_sha256(bytes: Option<&[u8]>) -> String {
    let mut hasher = Sha256::new();
    if let Some(b) = bytes {
        hasher.update(b);
    } else {
        hasher.update(b"");
    };
    let hash_str = hasher.finalize();
    hex::encode(hash_str)
}

pub fn sign_hmac_sha256(secret: &str, str_to_sign: &str) -> String {
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(str_to_sign.as_bytes());
    let res = mac.finalize().into_bytes();
    hex::encode(res)
}

pub fn generate_random_string(len: usize) -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

fn generate_nonce() -> String {
    generate_random_string(32)
}

// 签名入口
pub fn get_openapi_request_header<T: Serialize>(
    access_key_secret: &str,
    access_key_id: &str,
    sign_params: SignParams<T>,
) -> Result<(HashMap<String, String>, String), Error> {
    // region    --- sign authorization
    let (can_uri, url_, can_query_str) =
        generate_can_uri(sign_params.host, &sign_params.query_map, sign_params.style)?;

    let body_hash;
    let mut content_type = None;
    if let Some(pt) = sign_params.request_body {
        match pt {
            RequestBody::FormData(form_data) => {
                content_type = Some("application/x-www-form-urlencoded");
                let mut serializer = form_urlencoded::Serializer::new(String::new());
                for (k, v) in form_data.iter() {
                    serializer.append_pair(k, v);
                }
                // NOTE：这里有个问题，就是reqwest发送的时候对body编码使用的是serde_urlencoded，会不会和这里不一样？
                // 要是不一样签名就会失败
                let s = serializer.finish();
                // 选择使用文档中的ACS3-HMAC-SHA256算法
                // 文档对此部分的描述HashedRequestPayload，在这里这样实现：
                // 在OpenAPI元数据中，如果API的请求参数信息包含了"in": "body"或"in": "formData"时，需通过RequestBody传递参数
                // 如果传入的body_bytes为None则返回空字符串的sha256值，如果有body_bytes则计算其sha256值
                body_hash = hash_sha256(Some(s.as_bytes()));
            }
            RequestBody::Json(s) => {
                content_type = Some("application/json");
                body_hash = hash_sha256(Some(s.as_bytes()));
            }
            RequestBody::Binary(bytes) => {
                content_type = Some("application/octet-stream");
                body_hash = hash_sha256(Some(bytes));
            }
        };
    } else {
        body_hash = hash_sha256(None);
    }

    let generate_can_headers_res = generate_can_headers(
        sign_params.host,
        sign_params.x_acs_action,
        sign_params.x_acs_version,
        sign_params.x_acs_security_token,
        &body_hash,
        content_type,
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
    // println!("can_req_str:\n{}", can_req_str);
    let hashed_can_request = hash_sha256(Some(can_req_str.as_bytes()));
    let str_to_sign = format!("ACS3-HMAC-SHA256\n{hashed_can_request}");
    // println!("str_to_sign:\n{}", str_to_sign);
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

/// 按照签名文档要求序列化请求参数为BTreeMap<String, String>
///
/// 传入的query需要是一个to_value后为Value::Object的类型
pub fn to_query_map(query: impl Serialize) -> BTreeMap<String, String> {
    let v = serde_json::to_value(query).expect("Serialize to serde_json::Value failed");
    if !v.is_object() {
        panic!("to_query_map: input query is not an object!");
    }
    flatten_root(&v)
}

fn flatten_root(v: &Value) -> BTreeMap<String, String> {
    let mut res = BTreeMap::new();

    if let Value::Object(map) = v {
        for (k, val) in map {
            flatten_with_prefix(k, val, &mut res);
        }
    }

    res
}

fn flatten_with_prefix(prefix: &str, v: &Value, out: &mut BTreeMap<String, String>) {
    match v {
        Value::Null => {
            out.insert(prefix.to_owned(), "".to_owned());
        }
        Value::Bool(b) => {
            out.insert(prefix.to_owned(), b.to_string());
        }
        Value::Number(n) => {
            out.insert(prefix.to_owned(), n.to_string());
        }
        Value::String(s) => {
            out.insert(prefix.to_owned(), s.clone());
        }
        Value::Array(arr) => {
            for (i, item) in arr.iter().enumerate() {
                // 下标从 1 开始："Tag.1"、"Tag.2"…
                let new_prefix = format!("{prefix}.{}", i + 1);
                flatten_with_prefix(&new_prefix, item, out);
            }
        }
        Value::Object(map) => {
            for (k, val) in map {
                let new_prefix = format!("{prefix}.{k}");
                flatten_with_prefix(&new_prefix, val, out);
            }
        }
    }
}
