use reqwest::StatusCode;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("params error: {0}")]
    Common(String),
    #[error("request failed: code: {code}\nbody: {body}")]
    API { code: StatusCode, body: String },
    #[error("use reqwest error:\n {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("inner error: {0}")]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}
