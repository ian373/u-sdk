#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("error: {0}")]
    Common(String),
    #[error("request API failed: {status}, message: {message}")]
    RequestAPIFailed { status: String, message: String },
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
}

impl From<common_lib::Error> for Error {
    fn from(e: common_lib::Error) -> Self {
        match e {
            common_lib::Error::Common(msg) => Error::Common(msg),
            common_lib::Error::RequestAPIFailed { status, message } => {
                Error::RequestAPIFailed { status, message }
            }
            common_lib::Error::Reqwest(e) => Error::Reqwest(e),
        }
    }
}
