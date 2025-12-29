use rand::{TryRngCore, rand_core::OsError, rngs::OsRng};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("OS random number generator error")]
    OsRandom(#[from] OsError),

    #[error("Argon2 password hash error: {0}")]
    PasswordHashing(#[from] argon2::password_hash::Error),

    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("Invalid token format")]
    InvalidTokenFormat,

    #[error("Invalid token length")]
    InvalidTokenLength,
}

pub type Result<T> = core::result::Result<T, Error>;
