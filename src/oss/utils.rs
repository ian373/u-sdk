use crate::error::Error;
use base64::{engine::general_purpose, Engine};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::Serialize;
use std::collections::HashMap;

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
pub(crate) trait SerializeToHashMap
where
    Self: Sized + Serialize,
{
    fn serialize_to_hashmap(&self) -> Result<HashMap<String, String>, Error> {
        let r = serde_json::from_value(serde_json::to_value(self)?)?;
        Ok(r)
    }
}
