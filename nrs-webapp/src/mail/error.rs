use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Resend error: {0}")]
    Resend(#[from] resend_rs::Error),
}

pub type Result<T> = core::result::Result<T, Error>;
