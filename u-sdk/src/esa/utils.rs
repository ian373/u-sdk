use super::Error;
use serde::de::IntoDeserializer;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

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
/// 1. 字段为空字符串 `""` 时，反序列化为 `None`
/// 2. 字段为非空字符串时，尝试将其反序列化为 T
///
/// 在这个函数中字段不可能不存在，因为字段不存在时 serde 不会进入此函数
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

/// 只接受 JSON 对象，如果是空对象 `{}` 则返回 None，否则反序列化为 T
pub fn de_non_empty_object<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: for<'a> Deserialize<'a>,
{
    let value = Value::deserialize(deserializer)?;
    match value {
        // 如果是空对象 {} → None
        Value::Object(map) if map.is_empty() => Ok(None),
        // 如果是非空对象 → 反序列化为 T
        Value::Object(_) => T::deserialize(value)
            .map(Some)
            .map_err(serde::de::Error::custom),
        // 其他 JSON 类型（如字符串、数组等）→ 错误
        _ => Err(serde::de::Error::custom(
            "expected JSON object for this field",
        )),
    }
}

pub fn se_as_json_string<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: Serialize,
    S: Serializer,
{
    let json_str = serde_json::to_string(&value).map_err(serde::ser::Error::custom)?;
    serializer.serialize_str(&json_str)
}
