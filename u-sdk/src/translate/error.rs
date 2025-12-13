#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("request api failed:\n{code}\nmessage: {message}")]
    RequestAPIFailed { code: String, message: String },
    #[error("error: {0}")]
    Common(String),
    #[error("reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("error: {0}")]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}
