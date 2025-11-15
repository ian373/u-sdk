use crate::Error;
use hmac::{Hmac, Mac};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use sha1::Sha1;
use std::collections::HashMap;
use time::OffsetDateTime;
use time::format_description::well_known::iso8601::{
    Config, EncodedConfig, Iso8601, TimePrecision,
};

/// 输出格式: Day, DD Mon YYYY hh:mm:ss GMT
///
/// eg: Thu, 13 Nov 2025 13:32:03 GMT
//  TODO 似乎不需要替换+0000为GMT，可以直接使用Rfc2822格式，测试一下到时后
pub fn gmt_format(date_time: &OffsetDateTime) -> String {
    use time::format_description::well_known::Rfc2822;
    date_time.format(&Rfc2822).unwrap().replace("+0000", "GMT")
}

/// 输出格式: YYYY-MM-DDThh:mm:ssZ
///
/// eg: 2025-11-13T13:31:09Z
//  TODO 似乎不需要把精度设置为秒，可以直接使用默认的ISO 8601格式，测试一下到时后
pub fn now_iso8601() -> String {
    const ENCODED_CONFIG: EncodedConfig = Config::DEFAULT
        .set_time_precision(TimePrecision::Second {
            decimal_digits: None,
        })
        .encode();

    OffsetDateTime::now_utc()
        .format(&Iso8601::<ENCODED_CONFIG>)
        .unwrap()
}

pub fn into_header_map(map: HashMap<String, String>) -> HeaderMap {
    map.iter()
        .map(|(k, v)| {
            let name = HeaderName::from_bytes(k.as_bytes()).unwrap();
            let value = HeaderValue::from_bytes(v.as_bytes()).unwrap();
            (name, value)
        })
        .collect()
}

pub fn sign_hmac_sha1(secret: &str, str_to_sign: &str) -> Vec<u8> {
    type HmacSha1 = Hmac<Sha1>;
    let mut mac = HmacSha1::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(str_to_sign.as_bytes());
    mac.finalize().into_bytes().to_vec()
}

pub async fn into_request_failed_error(resp: reqwest::Response) -> Error {
    let status = resp.status();
    let body = resp.text().await;
    match body {
        Ok(message) => Error::RequestAPIFailed {
            status: status.to_string(),
            message,
        },
        Err(e) => Error::Reqwest(e),
    }
}

pub async fn parse_json_response<T: serde::de::DeserializeOwned>(
    resp: reqwest::Response,
) -> Result<T, Error> {
    let status = resp.status();

    if !status.is_success() {
        return Err(into_request_failed_error(resp).await);
    }

    let text = resp.text().await?;
    let data = serde_json::from_str(&text)
        .map_err(|e| Error::Common(format!("JSON parse error: {}", e)))?;
    Ok(data)
}

pub async fn parse_xml_response<T: serde::de::DeserializeOwned>(
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
