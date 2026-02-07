use thiserror::Error;

use crate::auth::external;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Login error: {0}")]
    Login(LoginError),

    #[error("Invalid user UUID: {0}")]
    UuidParseError(uuid::Error),

    #[error("Invalid URL: {0}")]
    UrlParseError(#[from] url::ParseError),

    #[error("Error from external authentication provider: {0}")]
    ExternalAuth(#[from] external::Error),
}

#[derive(Debug, Error)]
pub enum LoginError {
    #[error("Invalid credentials provided")]
    InvalidCredentials,
}

pub type Result<T> = core::result::Result<T, Error>;
