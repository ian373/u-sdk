//! 错误类型

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// 使用reqwest发出请求，如果发生错误，返回[reqwest::Error]
    #[error("request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    /// 解析json数据出现问题，可能是API返回的数据结构发生了变化，返回[serde_json::Error]
    #[error("json deserialize error, may be the api response is changed: {0}")]
    DeserializeFail(#[from] serde_json::Error),
    /// 出现该情况，一般认为是对API返回的xml数据反序列失败，说明代码中反序列的结构体缺少属性或存在错误属性，返回[quick_xml::DeError]
    #[cfg(feature = "oss")]
    #[error("[xml deserialize error]\nmsg:{source:?}\norigin_text:\n{origin_text}")]
    XMLDeError {
        source: quick_xml::DeError,
        origin_text: String,
    },
    /// 一般性错误，通常出现在使用`.unwrap()`的情况
    #[error("common error: {0}")]
    CommonError(String),
}
