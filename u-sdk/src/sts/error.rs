#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("error: {0}")]
    Common(String),
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("response status is not success: {status}, text: {text}")]
    RequestAPIFailed { status: String, text: String },
}
