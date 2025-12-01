use crate::credentials::CredentialsError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("error: {0}")]
    Common(String),
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("response status is not success: {status}, text: {text}")]
    RequestAPIFailed { status: String, text: String },
    #[error("io error: {0}")]
    IO(#[from] std::io::Error),
    #[error("credentials error: {0}")]
    Credentials(#[from] CredentialsError),
    #[error("error: {0}")]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}
