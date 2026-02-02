use oauth2::{ConfigurationError, RequestTokenError, basic::BasicRequestTokenError};
use openidconnect::{ClaimsVerificationError, DiscoveryError};
use thiserror::Error;

use crate::model::OAuth2HttpClientError;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Login error: {0}")]
    Login(LoginError),

    #[error("Invalid user UUID: {0}")]
    UuidParseError(uuid::Error),

    #[error("HTTP request error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("OAuth2/OIDC provider not found: {0}")]
    ProviderNotFound(String),

    #[error("OIDC discovery error: {0}")]
    OidcDiscovery(#[from] DiscoveryError<OAuth2HttpClientError>),

    #[error("OAuth2 configuration error: {0}")]
    OAuth2InvalidConfiguration(#[from] ConfigurationError),

    #[error("OAuth2 token exchange error: {0}")]
    TokenExchange(#[from] BasicRequestTokenError<OAuth2HttpClientError>),

    #[error("JSON serialization/deserialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("Auth flow state cookie not found")]
    AuthFlowStateCookieNotFound,

    #[error("Temp token cookie not found")]
    TempTokenCookieNotFound,

    #[error("Mismatched CSRF state in OAuth2 flow")]
    CsrfStateMismatch,

    #[error("Invalid ID token type")]
    InvalidIdTokenType,

    #[error("Invalid ID token claims: {0}")]
    InvalidIdTokenClaims(#[from] ClaimsVerificationError),

    #[error("Mismatched email")]
    EmailMismatch,
}

#[derive(Debug, Error)]
pub enum LoginError {
    #[error("Invalid credentials provided")]
    InvalidCredentials,
}

pub type Result<T> = core::result::Result<T, Error>;
