use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed creating DB connection pool: {0}")]
    FailedCreatingDbPool(String),
}

pub type Result<T> = core::result::Result<T, Error>;
