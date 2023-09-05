#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("reqwest error")]
    ReqwestError(#[from] reqwest::Error),
    #[error("response status code is not 200")]
    StatusCodeNot200Resp(reqwest::Response),
    /// 出现该情况，一般认为是对API返回的xml数据反序列失败，说明代码中反序列的结构体缺少属性或存在错误属性
    #[error("xml deserialize error")]
    XMLDeError(#[from] quick_xml::DeError),
    #[error("simple error")]
    CommonError(String),
}
