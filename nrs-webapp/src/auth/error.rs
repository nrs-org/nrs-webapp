use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Login error: {0}")]
    Login(LoginError),
}

#[derive(Debug, Error)]
pub enum LoginError {
    #[error("Invalid credentials provided")]
    InvalidCredentials,
}

pub type Result<T> = core::result::Result<T, Error>;
