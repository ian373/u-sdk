#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("error: {0}")]
    Common(String),
}
