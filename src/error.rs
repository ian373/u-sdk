//! 错误类型

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// 使用reqwest发出请求，如果发生错误，返回[reqwest::Error]
    #[error("reqwest error")]
    ReqwestError(#[from] reqwest::Error),
    /// 请求API成功，但是Status Code不是200，返回[reqwest::Response]
    #[error("response status code is not 200")]
    StatusCodeNot200Resp(reqwest::Response),
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
