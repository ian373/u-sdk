#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("error: {0}")]
    Common(String),
    #[error("request api failed: {status}, message: {message}")]
    RequestAPIFailed { status: String, message: String },
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
}
