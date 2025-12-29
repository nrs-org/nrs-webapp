use std::{fmt::Display, str::FromStr, sync::OnceLock};

use base64::{
    Engine,
    prelude::{BASE64_STANDARD, BASE64_URL_SAFE_NO_PAD},
};
use hmac::{Hmac, Mac};
use rand::{TryRngCore, rngs::OsRng};
use sha2::Sha256;

use super::{Error, Result};

pub const TOKEN_LENGTH: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token([u8; TOKEN_LENGTH]);

impl Token {
    /// Generates a cryptographically secure random token.
    ///
    /// Returns a newly generated `Token` on success; returns an error if the OS random
    /// source fails to provide enough entropy.
    ///
    /// # Examples
    ///
    /// ```
    /// let token = Token::generate().unwrap();
    /// let s = token.to_string();
    /// assert!(!s.is_empty());
    /// ```
    pub fn generate() -> Result<Self> {
        let mut rng = OsRng;
        let mut bytes = [0u8; TOKEN_LENGTH];
        rng.try_fill_bytes(&mut bytes)?;
        Ok(Self(bytes))
    }
}

impl Display for Token {
    /// Formats the token as URL-safe Base64 without padding.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::crypt::token::{Token, TOKEN_LENGTH};
    ///
    /// let token = Token([0u8; TOKEN_LENGTH]);
    /// let s = format!("{}", token);
    /// assert!(!s.is_empty());
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        BASE64_URL_SAFE_NO_PAD.encode(self.0).fmt(f)
    }
}

impl FromStr for Token {
    type Err = Error;

    /// Parses a `Token` from a URL-safe Base64 string without padding.
    ///
    /// `s` must be a URL-safe Base64 (no padding) encoding of exactly 32 bytes; on success this returns the corresponding `Token`.
    ///
    /// # Errors
    ///
    /// - `Error::InvalidTokenFormat` if `s` is not valid URL-safe Base64 without padding.
    /// - `Error::InvalidTokenLength` if the decoded byte sequence is not exactly 32 bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    /// // round-trip via Display/FromStr
    /// let original = Token::generate().unwrap();
    /// let s = original.to_string();
    /// let parsed = Token::from_str(&s).unwrap();
    /// assert_eq!(parsed.to_string(), s);
    /// ```
    fn from_str(s: &str) -> Result<Self> {
        BASE64_URL_SAFE_NO_PAD
            .decode(s)
            .map_err(|_| Error::InvalidTokenFormat)
            .and_then(|bytes| {
                if bytes.len() != TOKEN_LENGTH {
                    return Err(Error::InvalidTokenLength);
                }
                let mut array = [0u8; TOKEN_LENGTH];
                array.copy_from_slice(&bytes);
                Ok(Token(array))
            })
    }
}

pub struct TokenHasher(Hmac<Sha256>);

impl TokenHasher {
    /// Creates a `TokenHasher` backed by HMAC-SHA256 using the provided secret key.
    ///
    /// The provided `secret` is used as the HMAC key; returns a `TokenHasher` on success or an error
    /// if the key length is invalid for the underlying HMAC implementation.
    ///
    /// # Examples
    ///
    /// ```
    /// let hasher = nrs_webapp::crypt::token::TokenHasher::new(b"my-secret-key").unwrap();
    /// let token = nrs_webapp::crypt::token::Token::generate().unwrap();
    /// let _digest = hasher.hash(&token);
    /// ```
    pub fn new(secret: &[u8]) -> anyhow::Result<Self> {
        Ok(Self(Hmac::new_from_slice(secret)?))
    }

    /// Returns the global `TokenHasher` initialized from application configuration.
    ///
    /// Lazily constructs a static `TokenHasher` from `AppConfig::get().SERVICE_TOKEN_SECRET` and returns a `'static` reference to it. This function will panic if the configured secret cannot be used to create a `TokenHasher`.
    ///
    /// # Examples
    ///
    /// ```
    /// let hasher = crate::crypt::token::TokenHasher::get_from_config();
    /// let token = crate::crypt::token::Token::generate().unwrap();
    /// let digest = hasher.hash(&token);
    /// assert!(!digest.is_empty());
    /// ```
    pub fn get_from_config() -> &'static Self {
        static HASHER: OnceLock<TokenHasher> = OnceLock::new();
        HASHER.get_or_init(|| {
            let config = crate::config::AppConfig::get();
            TokenHasher::new(&config.SERVICE_TOKEN_SECRET)
                .expect("should not fail with valid secret")
        })
    }

    /// Computes the HMAC-SHA256 of a token and returns it as a standard Base64 string.
    ///
    /// Returns the Base64 (standard alphabet) encoding of the HMAC-SHA256 digest computed
    /// over the token's 32-byte payload using this hasher's secret.
    ///
    /// # Examples
    ///
    /// ```
    /// use nrs_webapp::crypt::token::{Token, TokenHasher};
    ///
    /// let token = Token::generate().unwrap();
    /// let hasher = TokenHasher::new(b"my-secret").unwrap();
    /// let hash_str = hasher.hash(&token);
    /// assert!(!hash_str.is_empty());
    /// ```
    pub fn hash(&self, token: &Token) -> String {
        let mut mac = self.0.clone();
        mac.update(&token.0);
        let result = mac.finalize();
        let code_bytes = result.into_bytes();
        BASE64_STANDARD.encode(code_bytes)
    }
}
