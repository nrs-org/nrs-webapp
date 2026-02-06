use rand::rand_core::OsError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("OS random number generator error")]
    OsRandom(#[from] OsError),

    #[error("Argon2 password hash error: {0}")]
    PasswordHashing(#[from] argon2::password_hash::Error),

    #[error("AES-GCM encryption/decryption error")]
    AesGcm,

    #[error("Invalid token format")]
    InvalidTokenFormat,

    #[error("Invalid token length")]
    InvalidTokenLength,

    #[error("Invalid key length")]
    InvalidKeyLength,

    #[error("Token has expired")]
    TokenExpired,

    #[error("Ciphertext too short")]
    CiphertextTooShort,
}

pub type Result<T> = core::result::Result<T, Error>;

impl From<aes_gcm::Error> for Error {
    fn from(_: aes_gcm::Error) -> Self {
        // the error type is opaque to prevent side-channel attacks
        Self::AesGcm
    }
}
