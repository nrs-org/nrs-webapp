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
        BASE64_URL_SAFE_NO_PAD.encode(&self.0).fmt(f)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> Config {
        Config {
            token_secret: "test_token_secret_key_for_hmac_must_be_long_enough_for_security".to_string(),
            email_verification_expiry_secs: 86400, // 24 hours
            password_reset_expiry_secs: 900, // 15 minutes
            ..Default::default()
        }
    }

    #[test]
    fn test_generate_email_verification_token() {
        let config = test_config();
        let user_id = 42;
        
        let result = generate_email_verification_token(user_id, &config);
        assert!(result.is_ok(), "Token generation should succeed");
        
        let token = result.unwrap();
        assert!(!token.is_empty(), "Token should not be empty");
        // Base64 encoded HMAC-SHA256 should be around 44 characters
        assert!(token.len() > 20, "Token should have reasonable length");
    }

    #[test]
    fn test_generate_password_reset_token() {
        let config = test_config();
        let user_id = 42;
        
        let result = generate_password_reset_token(user_id, &config);
        assert!(result.is_ok(), "Token generation should succeed");
        
        let token = result.unwrap();
        assert!(!token.is_empty(), "Token should not be empty");
    }

    #[test]
    fn test_verify_email_verification_token_success() {
        let config = test_config();
        let user_id = 42;
        
        let token = generate_email_verification_token(user_id, &config).unwrap();
        let result = verify_email_verification_token(&token, user_id, &config);
        
        assert!(result.is_ok(), "Token verification should succeed");
        assert!(result.unwrap(), "Token should be valid");
    }

    #[test]
    fn test_verify_password_reset_token_success() {
        let config = test_config();
        let user_id = 42;
        
        let token = generate_password_reset_token(user_id, &config).unwrap();
        let result = verify_password_reset_token(&token, user_id, &config);
        
        assert!(result.is_ok(), "Token verification should succeed");
        assert!(result.unwrap(), "Token should be valid");
    }

    #[test]
    fn test_verify_token_wrong_user_id() {
        let config = test_config();
        let user_id = 42;
        let wrong_user_id = 99;
        
        let token = generate_email_verification_token(user_id, &config).unwrap();
        let result = verify_email_verification_token(&token, wrong_user_id, &config);
        
        assert!(result.is_ok());
        assert!(!result.unwrap(), "Token should not be valid for different user");
    }

    #[test]
    fn test_verify_token_invalid_format() {
        let config = test_config();
        let invalid_token = "invalid_base64_!@#$";
        
        let result = verify_email_verification_token(invalid_token, 42, &config);
        assert!(result.is_err(), "Invalid token format should error");
    }

    #[test]
    fn test_verify_token_empty_string() {
        let config = test_config();
        
        let result = verify_email_verification_token("", 42, &config);
        assert!(result.is_err(), "Empty token should error");
    }

    #[test]
    fn test_verify_token_modified() {
        let config = test_config();
        let user_id = 42;
        
        let mut token = generate_email_verification_token(user_id, &config).unwrap();
        
        // Modify the token slightly
        if let Some(last_char) = token.pop() {
            token.push(if last_char == 'a' { 'b' } else { 'a' });
        }
        
        let result = verify_email_verification_token(&token, user_id, &config);
        assert!(result.is_ok());
        assert!(!result.unwrap(), "Modified token should not verify");
    }

    #[test]
    fn test_generate_tokens_different_for_same_user() {
        let config = test_config();
        let user_id = 42;
        
        let token1 = generate_email_verification_token(user_id, &config).unwrap();
        let token2 = generate_email_verification_token(user_id, &config).unwrap();
        
        // Tokens should be different due to timestamp
        assert_ne!(token1, token2, "Tokens generated at different times should differ");
    }

    #[test]
    fn test_generate_tokens_different_users() {
        let config = test_config();
        
        let token1 = generate_email_verification_token(1, &config).unwrap();
        let token2 = generate_email_verification_token(2, &config).unwrap();
        
        assert_ne!(token1, token2, "Tokens for different users should differ");
    }

    #[test]
    fn test_verify_token_wrong_secret() {
        let config = test_config();
        let user_id = 42;
        
        let token = generate_email_verification_token(user_id, &config).unwrap();
        
        let wrong_config = Config {
            token_secret: "completely_different_secret_key_for_testing_purposes".to_string(),
            ..config
        };
        
        let result = verify_email_verification_token(&token, user_id, &wrong_config);
        assert!(result.is_ok());
        assert!(!result.unwrap(), "Token should not verify with wrong secret");
    }

    #[test]
    fn test_generate_token_boundary_user_ids() {
        let config = test_config();
        
        // Test with 0
        let token = generate_email_verification_token(0, &config).unwrap();
        assert!(verify_email_verification_token(&token, 0, &config).unwrap());
        
        // Test with large ID
        let large_id = i64::MAX;
        let token = generate_email_verification_token(large_id, &config).unwrap();
        assert!(verify_email_verification_token(&token, large_id, &config).unwrap());
    }

    #[test]
    fn test_password_reset_vs_email_verification_tokens_different() {
        let config = test_config();
        let user_id = 42;
        
        let email_token = generate_email_verification_token(user_id, &config).unwrap();
        let reset_token = generate_password_reset_token(user_id, &config).unwrap();
        
        // These should be different types of tokens
        assert_ne!(email_token, reset_token, "Different token types should produce different values");
    }

    #[test]
    fn test_cross_verify_tokens_fail() {
        let config = test_config();
        let user_id = 42;
        
        let email_token = generate_email_verification_token(user_id, &config).unwrap();
        
        // Try to verify email token as password reset token
        let result = verify_password_reset_token(&email_token, user_id, &config);
        assert!(result.is_ok());
        assert!(!result.unwrap(), "Email token should not verify as password reset token");
    }

    #[test]
    fn test_verify_token_truncated() {
        let config = test_config();
        let user_id = 42;
        
        let token = generate_email_verification_token(user_id, &config).unwrap();
        let truncated = &token[..token.len() - 5];
        
        let result = verify_email_verification_token(truncated, user_id, &config);
        assert!(result.is_err() || !result.unwrap(), "Truncated token should not verify");
    }

    #[test]
    fn test_verify_token_with_padding_issues() {
        let config = test_config();
        
        // Test with various invalid base64 strings
        let invalid_tokens = vec![
            "abc",
            "abcd=",
            "====",
            "ab cd",
        ];
        
        for token in invalid_tokens {
            let result = verify_email_verification_token(token, 42, &config);
            assert!(result.is_err() || !result.unwrap(), "Invalid token '{}' should not verify", token);
        }
    }

    #[test]
    fn test_token_generation_consistent_encoding() {
        let config = test_config();
        let user_id = 42;
        
        let token = generate_email_verification_token(user_id, &config).unwrap();
        
        // Token should be valid base64
        assert!(base64::Engine::decode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, &token).is_ok(), 
                "Token should be valid base64");
    }

    #[test]
    fn test_verify_email_verification_token_rapid_succession() {
        let config = test_config();
        let user_id = 42;
        
        let token1 = generate_email_verification_token(user_id, &config).unwrap();
        let token2 = generate_email_verification_token(user_id, &config).unwrap();
        
        // Both tokens should verify correctly
        assert!(verify_email_verification_token(&token1, user_id, &config).unwrap());
        assert!(verify_email_verification_token(&token2, user_id, &config).unwrap());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_generate_success() {
        let result = Token::generate();
        assert!(result.is_ok(), "Token generation should succeed");
    }

    #[test]
    fn test_token_generate_uniqueness() {
        let token1 = Token::generate().unwrap();
        let token2 = Token::generate().unwrap();
        
        assert_ne!(
            token1.to_string(),
            token2.to_string(),
            "Generated tokens should be unique"
        );
    }

    #[test]
    fn test_token_display_not_empty() {
        let token = Token::generate().unwrap();
        let display = token.to_string();
        
        assert!(!display.is_empty(), "Token display should not be empty");
        assert!(display.len() > 40, "Base64-encoded 32 bytes should be ~44 chars");
    }

    #[test]
    fn test_token_from_str_roundtrip() {
        let original = Token::generate().unwrap();
        let string = original.to_string();
        let parsed = Token::from_str(&string).unwrap();
        
        assert_eq!(original, parsed, "Roundtrip should preserve token");
        assert_eq!(string, parsed.to_string(), "String representation should match");
    }

    #[test]
    fn test_token_from_str_invalid_format() {
        let invalid_tokens = vec![
            "not-base64-!@#$",
            "short",
            "",
            "AAAA",
        ];
        
        for invalid in invalid_tokens {
            let result = Token::from_str(invalid);
            assert!(result.is_err(), "Invalid token '{}' should fail parsing", invalid);
        }
    }

    #[test]
    fn test_token_from_str_invalid_length() {
        // Valid base64 but wrong length
        let too_short = BASE64_URL_SAFE_NO_PAD.encode(&[0u8; 16]);
        let too_long = BASE64_URL_SAFE_NO_PAD.encode(&[0u8; 64]);
        
        assert!(Token::from_str(&too_short).is_err(), "Short token should fail");
        assert!(Token::from_str(&too_long).is_err(), "Long token should fail");
    }

    #[test]
    fn test_token_hasher_new() {
        let secret = b"test-secret-key";
        let result = TokenHasher::new(secret);
        
        assert!(result.is_ok(), "TokenHasher creation should succeed");
    }

    #[test]
    fn test_token_hasher_hash_consistency() {
        let secret = b"test-secret";
        let hasher = TokenHasher::new(secret).unwrap();
        let token = Token::generate().unwrap();
        
        let hash1 = hasher.hash(&token);
        let hash2 = hasher.hash(&token);
        
        assert_eq!(hash1, hash2, "Same token should produce same hash");
    }

    #[test]
    fn test_token_hasher_different_tokens_different_hashes() {
        let secret = b"test-secret";
        let hasher = TokenHasher::new(secret).unwrap();
        
        let token1 = Token::generate().unwrap();
        let token2 = Token::generate().unwrap();
        
        let hash1 = hasher.hash(&token1);
        let hash2 = hasher.hash(&token2);
        
        assert_ne!(hash1, hash2, "Different tokens should produce different hashes");
    }

    #[test]
    fn test_token_hasher_different_secrets() {
        let token = Token::generate().unwrap();
        
        let hasher1 = TokenHasher::new(b"secret1").unwrap();
        let hasher2 = TokenHasher::new(b"secret2").unwrap();
        
        let hash1 = hasher1.hash(&token);
        let hash2 = hasher2.hash(&token);
        
        assert_ne!(hash1, hash2, "Different secrets should produce different hashes");
    }

    #[test]
    fn test_token_hasher_hash_is_base64() {
        let hasher = TokenHasher::new(b"secret").unwrap();
        let token = Token::generate().unwrap();
        let hash = hasher.hash(&token);
        
        let decoded = BASE64_STANDARD.decode(&hash);
        assert!(decoded.is_ok(), "Hash should be valid base64");
        assert_eq!(decoded.unwrap().len(), 32, "HMAC-SHA256 produces 32 bytes");
    }

    #[test]
    fn test_token_clone() {
        let token1 = Token::generate().unwrap();
        let token2 = token1.clone();
        
        assert_eq!(token1, token2, "Cloned token should equal original");
    }

    #[test]
    fn test_token_debug() {
        let token = Token::generate().unwrap();
        let debug = format!("{:?}", token);
        
        assert!(debug.contains("Token"), "Debug output should mention Token");
    }

    #[test]
    fn test_token_length_constant() {
        assert_eq!(TOKEN_LENGTH, 32, "Token length should be 32 bytes");
    }

    #[test]
    fn test_token_eq() {
        let bytes = [42u8; TOKEN_LENGTH];
        let token1 = Token(bytes);
        let token2 = Token(bytes);
        
        assert_eq!(token1, token2, "Tokens with same bytes should be equal");
    }

    #[test]
    fn test_token_ne() {
        let token1 = Token::generate().unwrap();
        let token2 = Token::generate().unwrap();
        
        assert_ne!(token1, token2, "Different random tokens should not be equal");
    }
}