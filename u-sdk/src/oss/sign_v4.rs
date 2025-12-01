use crate::oss::utils::{get_date_str, get_date_time_str};
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};
use time::OffsetDateTime;
use time::macros::format_description;
use u_sdk_common::helper::gmt_format;
use url::Url;
// 签名文档：https://help.aliyun.com/zh/oss/developer-reference/recommend-to-use-signature-version-4

pub(crate) enum HTTPVerb {
    Get,
    Put,
    Post,
    Delete,
    Head,
}
impl Display for HTTPVerb {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            HTTPVerb::Get => write!(f, "GET"),
            HTTPVerb::Put => write!(f, "PUT"),
            HTTPVerb::Post => write!(f, "POST"),
            HTTPVerb::Delete => write!(f, "DELETE"),
            HTTPVerb::Head => write!(f, "HEAD"),
        }
    }
}

// 构造过程和要求参考api文档中的[步骤1：构造CanonicalRequest]部分
// 对于canonical_header的构造，对[k:v]有一些要求（如必须存在的header，存在则加入签名的header，等等）
// 这里会保证k是小写的，其它要求不会进行检查，由调用者保证
fn get_canonical_request(
    http_verb: HTTPVerb,
    uri: &Url,
    bucket: Option<&str>,
    canonical_header: &BTreeMap<&str, &str>,
    additional_header: Option<&BTreeSet<&str>>,
) -> String {
    let canonical_uri = if let Some(bucket) = bucket {
        let mut s = url::form_urlencoded::byte_serialize(bucket.as_bytes()).collect::<String>();
        s.insert(0, '/');
        s.push_str(uri.path()); // uri.path即为object的名称，同时，要求path不能以/结尾
        s
    } else {
        "/".to_owned()
    };
    let canonical_query_string = uri
        .query_pairs()
        .map(|(k, v)| {
            let encoded_k = url::form_urlencoded::byte_serialize(k.as_bytes()).collect::<String>();
            if v.is_empty() {
                (encoded_k, "".to_string())
            } else {
                let encoded_v =
                    url::form_urlencoded::byte_serialize(v.as_bytes()).collect::<String>();
                (encoded_k, encoded_v)
            }
        })
        .collect::<BTreeMap<_, _>>()
        .into_iter()
        .map(|(k, v)| {
            if v.is_empty() {
                k
            } else {
                format!("{}={}", k, v)
            }
        })
        .collect::<Vec<_>>()
        .join("&");
    let mut canonical_headers_str = String::new();
    for (k, v) in canonical_header {
        canonical_headers_str.push_str(&format!("{}:{}\n", k.to_lowercase(), v.trim()));
    }

    let additional_header_str = if let Some(additional_header) = additional_header {
        additional_header
            .iter()
            .map(|k| k.to_lowercase())
            .collect::<Vec<_>>()
            .join(";")
    } else {
        "".to_owned()
    };

    let res = format!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        http_verb,
        canonical_uri,
        canonical_query_string,
        canonical_headers_str,
        additional_header_str,
        "UNSIGNED-PAYLOAD"
    );

    res
}

fn sign_hmac_sha256_byte(secret: &[u8], str_to_sign: &[u8]) -> Vec<u8> {
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(secret).unwrap();
    mac.update(str_to_sign);
    mac.finalize().into_bytes().to_vec()
}

pub(crate) struct SignV4Param<'a> {
    pub signing_region: &'a str,
    pub http_verb: HTTPVerb,
    pub uri: &'a Url,
    // 如果host中有bucket，则此处传入bucket名称，用于构建canonical_uri
    pub bucket: Option<&'a str>,
    pub header_map: &'a BTreeMap<&'a str, &'a str>,
    pub additional_header: Option<&'a BTreeSet<&'a str>>,
    pub date_time: &'a OffsetDateTime,
}

/// verb: GET, PUT, POST, DELETE...
/// uri like: "/", "/bucket/", "/bucket/object"; query: "xxx?xxx=xxx&xxx=xxx"
pub(crate) fn sign_v4(
    access_key_id: &str,
    access_key_secret: &str,
    sign_v4param: SignV4Param<'_>,
) -> String {
    let date_time = sign_v4param.date_time;
    let date = date_time
        .format(&format_description!("[year][month][day]"))
        .unwrap();
    let date_key = sign_hmac_sha256_byte(
        format!("aliyun_v4{}", access_key_secret).as_bytes(),
        date.as_bytes(),
    );
    let date_region_key = sign_hmac_sha256_byte(&date_key, sign_v4param.signing_region.as_bytes());
    let date_region_service_key = sign_hmac_sha256_byte(&date_region_key, b"oss");
    let signing_key = sign_hmac_sha256_byte(&date_region_service_key, b"aliyun_v4_request");
    let gmt = gmt_format(date_time);

    //region 构建string_to_sign
    let scope = format!(
        "{}/{}/{}",
        date, sign_v4param.signing_region, "oss/aliyun_v4_request"
    );
    let canonical_request_str = get_canonical_request(
        sign_v4param.http_verb,
        sign_v4param.uri,
        sign_v4param.bucket,
        sign_v4param.header_map,
        sign_v4param.additional_header,
    );
    // println!(
    //     "canonical_request_str:====\n{}\n====",
    //     canonical_request_str
    // );
    let mut hasher = Sha256::new();
    hasher.update(canonical_request_str.as_bytes());
    let hex_canonical_request = hex::encode(hasher.finalize());
    let string_to_sign = format!(
        "OSS4-HMAC-SHA256\n{}\n{}\n{}",
        gmt, scope, hex_canonical_request
    );
    // println!("string_to_sign:===========\n{}\n===========", string_to_sign);
    //endregion

    let signature = hex::encode(sign_hmac_sha256_byte(
        &signing_key,
        string_to_sign.as_bytes(),
    ));
    format!(
        "OSS4-HMAC-SHA256 Credential={}/{}/{}/oss/aliyun_v4_request, AdditionalHeaders=host, Signature={}",
        access_key_id, date, sign_v4param.signing_region, signature
    )
}

pub(crate) fn generate_v4_signature(
    access_key_secret: &str,
    sign_v4param: SignV4Param<'_>,
) -> String {
    let date_str = get_date_str(sign_v4param.date_time);
    let data_time_str = get_date_time_str(sign_v4param.date_time);
    let date_key = sign_hmac_sha256_byte(
        format!("aliyun_v4{}", access_key_secret).as_bytes(),
        date_str.as_bytes(),
    );
    let date_region_key = sign_hmac_sha256_byte(&date_key, sign_v4param.signing_region.as_bytes());
    let date_region_service_key = sign_hmac_sha256_byte(&date_region_key, b"oss");
    let signing_key = sign_hmac_sha256_byte(&date_region_service_key, b"aliyun_v4_request");

    //region 构建string_to_sign
    let scope = format!(
        "{}/{}/{}",
        date_str, sign_v4param.signing_region, "oss/aliyun_v4_request"
    );
    let canonical_request_str = get_canonical_request(
        sign_v4param.http_verb,
        sign_v4param.uri,
        sign_v4param.bucket,
        sign_v4param.header_map,
        sign_v4param.additional_header,
    );
    // println!(
    //     "canonical_request_str:====\n{}\n====",
    //     canonical_request_str
    // );
    let mut hasher = Sha256::new();
    hasher.update(canonical_request_str.as_bytes());
    let hex_canonical_request = hex::encode(hasher.finalize());
    let string_to_sign = format!(
        "OSS4-HMAC-SHA256\n{}\n{}\n{}",
        data_time_str, scope, hex_canonical_request
    );
    // println!(
    //     "string_to_sign:===========\n{}\n===========",
    //     string_to_sign
    // );
    //endregion

    hex::encode(sign_hmac_sha256_byte(
        &signing_key,
        string_to_sign.as_bytes(),
    ))
}
