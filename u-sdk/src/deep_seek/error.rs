#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("error: {0}")]
    Common(String),
    #[error("request API failed: {status}, message: {message}")]
    RequestAPIFailed { status: String, message: String },
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
}

impl From<u_sdk_common::Error> for Error {
    fn from(e: u_sdk_common::Error) -> Self {
        match e {
            u_sdk_common::Error::Common(msg) => Error::Common(msg),
            u_sdk_common::Error::RequestAPIFailed { status, message } => {
                Error::RequestAPIFailed { status, message }
            }
            u_sdk_common::Error::Reqwest(e) => Error::Reqwest(e),
        }
    }
}
