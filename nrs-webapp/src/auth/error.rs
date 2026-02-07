use thiserror::Error;

use crate::auth::external;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Login error: {0}")]
    Login(LoginError),

    #[error("Error from external authentication provider: {0}")]
    ExternalAuth(#[from] external::Error),
}

#[derive(Debug, Error)]
pub enum LoginError {
    #[error("Invalid credentials provided")]
    InvalidCredentials,
}

pub type Result<T> = core::result::Result<T, Error>;
