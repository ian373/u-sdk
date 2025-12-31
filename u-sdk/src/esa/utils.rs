use super::Error;
use serde::de::IntoDeserializer;
use serde::{Deserialize, Deserializer};

pub async fn parse_json_response<T: serde::de::DeserializeOwned>(
    resp: reqwest::Response,
) -> Result<T, Error> {
    let status = resp.status();

    if !status.is_success() {
        return Err(Error::RequestAPIFailed {
            code: status.to_string(),
            message: resp.text().await?,
        });
    }

    let data = resp.json().await?;
    Ok(data)
}

// 有些字段时在没有的情况下返回的json，该字段不是不存在，而是为空字符串`""`，需要特殊处理
// 这个方法处理这种情况
pub fn de_option_empty_string_as_none<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    // 先按 Option<serde_json::Value> 或 Option<String> 都行；
    // 用 Option<String> 的好处是明确只处理字符串场景
    // 处理字段不存在的情况
    let opt = Option::<String>::deserialize(deserializer)?;

    match opt {
        None => Ok(None),
        // 处理字段存在但值为空字符串的情况
        Some(s) if s.is_empty() => Ok(None),
        Some(s) => {
            // 关键点：把 String 再交给 T 的 Deserialize（复用 serde 对 enum 的规则）
            T::deserialize(s.into_deserializer()).map(Some)
        }
    }
}
