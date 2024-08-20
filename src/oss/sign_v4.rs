use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};
use time::OffsetDateTime;
use url::Url;

// 签名文档：https://help.aliyun.com/zh/oss/developer-reference/recommend-to-use-signature-version-4

pub(crate) enum HTTPVerb {
    Get,
    Put,
    Post,
    Delete,
}
impl Display for HTTPVerb {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            HTTPVerb::Get => write!(f, "GET"),
            HTTPVerb::Put => write!(f, "PUT"),
            HTTPVerb::Post => write!(f, "POST"),
            HTTPVerb::Delete => write!(f, "DELETE"),
        }
    }
}

fn get_canonical_request(
    http_verb: HTTPVerb,
    uri: &Url,
    canonical_header: &BTreeMap<&str, &str>,
    additional_header: Option<&BTreeSet<&str>>,
) -> String {
    let canonical_uri = url::form_urlencoded::byte_serialize(uri.path().as_bytes())
        .collect::<String>()
        .replace("%2F", "/");
    let canonical_query_string = uri
        .query_pairs()
        .collect::<BTreeMap<_, _>>()
        .iter()
        .map(|(k, v)| {
            if v.is_empty() {
                k.to_string()
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

    // println!("canonical_request:===========\n{}\n===========", res);
    res
}

fn string_to_sign(
    signing_region: &str,
    http_verb: HTTPVerb,
    uri: &Url,
    header_map: &BTreeMap<&str, &str>,
    additional_header: Option<&BTreeSet<&str>>,
    timestamp: &str,
    date: &str,
) -> String {
    let scope = format!("{}/{}/{}", date, signing_region, "oss/aliyun_v4_request");
    let canonical_request_str =
        get_canonical_request(http_verb, uri, header_map, additional_header);
    let mut hasher = Sha256::new();
    hasher.update(canonical_request_str.as_bytes());
    let hex_canonical_request = hex::encode(hasher.finalize());
    let res = format!(
        "OSS4-HMAC-SHA256\n{}\n{}\n{}",
        timestamp, scope, hex_canonical_request
    );
    // println!("string_to_sign:===========\n{}\n===========", res);
    res
}

fn sign_hmac_sha256_byte(secret: &[u8], str_to_sign: &[u8]) -> Vec<u8> {
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(secret).unwrap();
    mac.update(str_to_sign);
    mac.finalize().into_bytes().to_vec()
}

/// verb: GET, PUT, POST, DELETE...
/// uri like: "/", "/bucket/", "/bucket/object"; query: "xxx?xxx=xxx&xxx=xxx"
// 关于签名所必须的参数参考顶部签名文档，如canonical_header, additional_header等
#[allow(clippy::too_many_arguments)]
pub(crate) fn sign_v4(
    signing_region: &str,
    http_verb: HTTPVerb,
    uri: &Url,
    header_map: &BTreeMap<&str, &str>,
    addition_header: Option<&BTreeSet<&str>>,
    access_key_id: &str,
    secret_key: &str,
    date_time: &OffsetDateTime,
) -> String {
    let date = date_time
        .format(&time::format_description::parse("[year][month][day]").unwrap())
        .unwrap();
    let date_key = sign_hmac_sha256_byte(
        (format!("aliyun_v4{}", secret_key)).as_bytes(),
        date.as_bytes(),
    );
    let date_region_key = sign_hmac_sha256_byte(&date_key, signing_region.as_bytes());
    let date_region_service_key = sign_hmac_sha256_byte(&date_region_key, b"oss");
    let signing_key = sign_hmac_sha256_byte(&date_region_service_key, b"aliyun_v4_request");
    use time::format_description::well_known::Rfc2822;
    let gmt = date_time.format(&Rfc2822).unwrap().replace("+0000", "GMT");
    let string_to_sign = string_to_sign(
        signing_region,
        http_verb,
        uri,
        header_map,
        addition_header,
        &gmt,
        &date,
    );
    let signature = hex::encode(sign_hmac_sha256_byte(
        &signing_key,
        string_to_sign.as_bytes(),
    ));
    format!(
        "OSS4-HMAC-SHA256 Credential={}/{}/{}/oss/aliyun_v4_request, AdditionalHeaders=host, Signature={}",
        access_key_id, date, signing_region, signature
    )
}
