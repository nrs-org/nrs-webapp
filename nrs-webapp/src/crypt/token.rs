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
    pub fn generate() -> Result<Self> {
        let mut rng = OsRng;
        let mut bytes = [0u8; TOKEN_LENGTH];
        rng.try_fill_bytes(&mut bytes)?;
        Ok(Self(bytes))
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        BASE64_URL_SAFE_NO_PAD.encode(&self.0).fmt(f)
    }
}

impl FromStr for Token {
    type Err = Error;

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
    pub fn new(secret: &[u8]) -> anyhow::Result<Self> {
        Ok(Self(Hmac::new_from_slice(secret)?))
    }

    pub fn get_from_config() -> &'static Self {
        static HASHER: OnceLock<TokenHasher> = OnceLock::new();
        HASHER.get_or_init(|| {
            let config = crate::config::AppConfig::get();
            TokenHasher::new(&config.SERVICE_TOKEN_SECRET)
                .expect("should not fail with valid secret")
        })
    }

    pub fn hash(&self, token: &Token) -> String {
        let mut mac = self.0.clone();
        mac.update(&token.0);
        let result = mac.finalize();
        let code_bytes = result.into_bytes();
        BASE64_STANDARD.encode(code_bytes)
    }
}
