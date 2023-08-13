#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("reqwest error")]
    ReqwestError(#[from] reqwest::Error),
    #[error("response status code is not 200")]
    StatusCodeNot200Resp(reqwest::Response),
}
