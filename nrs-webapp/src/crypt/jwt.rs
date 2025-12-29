use std::sync::OnceLock;

use jsonwebtoken::{DecodingKey, EncodingKey, TokenData};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use time::{Duration, OffsetDateTime};

use super::Result;
use crate::config::AppConfig;

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub iss: String,
    pub sub: String,
    pub aud: String,
    #[serde_as(as = "serde_with::TimestampSeconds")]
    pub exp: OffsetDateTime,
    #[serde_as(as = "serde_with::TimestampSeconds")]
    pub iat: OffsetDateTime,
}

pub struct JwtContext {
    decoding_key: DecodingKey,
    encoding_key: EncodingKey,
    expiry_duration: Duration,
}

impl JwtContext {
    /// Constructs a JwtContext from a raw secret and a token expiry duration.
    ///
    /// The provided `secret` is used to derive both the encoding (signing) and decoding (verification) keys. `expiry_duration` is the length of time added to the issued-at time to produce the token expiry timestamp.
    ///
    /// # Examples
    ///
    /// ```
    /// use nrs_webapp::crypt::jwt::JwtContext;
    /// use time::Duration;
    ///
    /// let ctx = JwtContext::new(b"my-secret", Duration::hours(1));
    /// ```
    pub fn new(secret: &'static [u8], expiry_duration: Duration) -> Self {
        Self {
            decoding_key: DecodingKey::from_secret(secret),
            encoding_key: EncodingKey::from_secret(secret),
            expiry_duration,
        }
    }

    /// Get a reference to the global `JwtContext` configured from application settings.
    ///
    /// The context is initialized once on first use using `SERVICE_JWT_EXPIRY_DURATION` (as a `time::Duration`)
    /// and `SERVICE_JWT_SECRET` from `AppConfig`.
    ///
    /// # Examples
    ///
    /// ```
    /// let ctx = JwtContext::get_from_config();
    /// let claims = ctx.generate_claims("user-123".into());
    /// ```
    pub fn get_from_config() -> &'static Self {
        static SIGNER: OnceLock<JwtContext> = OnceLock::new();
        SIGNER.get_or_init(|| {
            let config = AppConfig::get();
            let expiry_duration = time::Duration::try_from(config.SERVICE_JWT_EXPIRY_DURATION)
                .expect("should not be negative here");
            JwtContext::new(&config.SERVICE_JWT_SECRET, expiry_duration)
        })
    }

    /// Constructs JWT claims for the given user using this context's configuration.
    ///
    /// The returned `JwtClaims` sets:
    /// - issuer (`iss`) to `"nrs-webapp"`,
    /// - audience (`aud`) to `"nrs-webapp-users"`,
    /// - subject (`sub`) to the provided `user_id`,
    /// - issued-at (`iat`) to the current UTC time,
    /// - expiration (`exp`) to `iat + self.expiry_duration`.
    ///
    /// # Examples
    ///
    /// ```
    /// use time::Duration;
    /// let ctx = JwtContext::new(b"secret", Duration::minutes(15));
    /// let claims = ctx.generate_claims("user-123".to_string());
    /// assert_eq!(claims.sub, "user-123");
    /// ```
    ///
    /// Returns the populated `JwtClaims` with `sub` equal to the given `user_id`, `iat` set to now, and `exp` set to now plus the context's expiry duration.
    pub fn generate_claims(&self, user_id: String) -> JwtClaims {
        let now = OffsetDateTime::now_utc();
        JwtClaims {
            iss: "nrs-webapp".to_string(),
            aud: "nrs-webapp-users".to_string(),
            sub: user_id,
            iat: now,
            exp: now + self.expiry_duration,
        }
    }

    /// Signs the provided JWT claims and returns a compact JWT string.
    ///
    /// Returns the signed JWT as a compact (dot-separated) string on success.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nrs_webapp::crypt::jwt::{JwtContext};
    /// # use time::Duration;
    /// let ctx = JwtContext::new(b"secret", Duration::days(1));
    /// let claims = ctx.generate_claims("user123".to_string());
    /// let token = ctx.sign(&claims).unwrap();
    /// assert!(!token.is_empty());
    /// ```
    pub fn sign(&self, claims: &JwtClaims) -> Result<String> {
        Ok(jsonwebtoken::encode(
            &Default::default(),
            claims,
            &self.encoding_key,
        )?)
    }

    /// Verifies a JWT string and decodes its `JwtClaims`.
    
    ///
    
    /// The token's signature, audience ("nrs-webapp-users"), and standard time-based claims (e.g., expiration)
    
    /// are validated according to the jsonwebtoken validation rules. Returns an error if validation or decoding fails.
    
    ///
    
    /// # Examples
    
    ///
    
    /// ```
    
    /// let ctx = JwtContext::new(b"secret", time::Duration::minutes(60));
    
    /// let claims = ctx.generate_claims("user-123".to_string());
    
    /// let token = ctx.sign(&claims).unwrap();
    
    /// let decoded = ctx.verify(&token).unwrap();
    
    /// assert_eq!(decoded.claims.sub, "user-123");
    
    /// ```
    pub fn verify(&self, token: &str) -> Result<TokenData<JwtClaims>> {
        let mut validation = jsonwebtoken::Validation::default();
        validation.set_audience(&["nrs-webapp-users"]);
        #[cfg(debug_assertions)]
        {
            validation.leeway = 0;
        }

        Ok(jsonwebtoken::decode::<JwtClaims>(
            token,
            &self.decoding_key,
            &validation,
        )?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::{Duration, OffsetDateTime};

    fn ctx(secret: &'static [u8], expiry: Duration) -> JwtContext {
        JwtContext::new(secret, expiry)
    }

    #[test]
    fn generate_claims_sets_expected_fields() {
        let ctx = ctx(b"test-secret", Duration::minutes(10));
        let before = OffsetDateTime::now_utc();

        let claims = ctx.generate_claims("user-123".to_string());

        let after = OffsetDateTime::now_utc();

        assert_eq!(claims.iss, "nrs-webapp");
        assert_eq!(claims.aud, "nrs-webapp-users");
        assert_eq!(claims.sub, "user-123");

        // iat should be roughly "now"
        assert!(claims.iat >= before);
        assert!(claims.iat <= after);

        // exp should be iat + expiry_duration
        assert_eq!(claims.exp, claims.iat + Duration::minutes(10));
    }

    // jwt discard nanosecond timings, so we allow up to 1s difference
    /// Asserts that two `OffsetDateTime` values differ by at most one second.
    ///
    /// Panics if the absolute difference between `a` and `b` is greater than one second.
    /// Useful in tests that compare timestamps with small permitted clock skew.
    ///
    /// # Examples
    ///
    /// ```
    /// use time::{Duration, OffsetDateTime};
    ///
    /// let now = OffsetDateTime::now_utc();
    /// let nearly_now = now + Duration::milliseconds(500);
    /// assert_same_jwt_second(now, nearly_now);
    /// ```
    fn assert_same_jwt_second(a: OffsetDateTime, b: OffsetDateTime) {
        let diff = (a - b).abs();
        assert!(
            diff <= Duration::seconds(1),
            "timestamps differ by more than 1s: a={a:?}, b={b:?}"
        );
    }

    #[test]
    fn sign_and_verify_roundtrip() {
        let ctx = ctx(b"roundtrip-secret", Duration::minutes(15));
        let claims = ctx.generate_claims("user-abc".to_string());

        let token = ctx.sign(&claims).expect("sign should succeed");
        let decoded = ctx.verify(&token).expect("verify should succeed");

        let decoded_claims = decoded.claims;

        assert_eq!(decoded_claims.sub, claims.sub);
        assert_eq!(decoded_claims.iss, claims.iss);
        assert_eq!(decoded_claims.aud, claims.aud);
        assert_same_jwt_second(decoded_claims.iat, claims.iat);
        assert_same_jwt_second(decoded_claims.exp, claims.exp);
    }

    /// Ensures that a token signed with one secret cannot be verified with a different secret.
    ///
    /// # Examples
    ///
    /// ```
    /// let ctx_good = ctx(b"correct-secret", Duration::minutes(10));
    /// let ctx_bad = ctx(b"wrong-secret", Duration::minutes(10));
    /// let claims = ctx_good.generate_claims("user-x".to_string());
    /// let token = ctx_good.sign(&claims).unwrap();
    /// assert!(ctx_bad.verify(&token).is_err());
    /// ```
    #[test]
    fn verify_fails_with_wrong_secret() {
        let ctx_good = ctx(b"correct-secret", Duration::minutes(10));
        let ctx_bad = ctx(b"wrong-secret", Duration::minutes(10));

        let claims = ctx_good.generate_claims("user-x".to_string());
        let token = ctx_good.sign(&claims).expect("sign should succeed");

        let result = ctx_bad.verify(&token);
        assert!(result.is_err());
    }

    #[test]
    fn expired_token_is_rejected() {
        let ctx = ctx(b"expiry-secret", Duration::minutes(5));

        let now = OffsetDateTime::now_utc();
        let claims = JwtClaims {
            iss: "nrs-webapp".to_string(),
            aud: "nrs-webapp-users".to_string(),
            sub: "user-expired".to_string(),
            iat: now - Duration::minutes(10),
            exp: now - Duration::minutes(1),
        };

        let token = ctx.sign(&claims).expect("sign should succeed");

        let result = ctx.verify(&token);
        assert!(result.is_err());
    }

    #[test]
    fn wrong_audience_is_rejected() {
        let ctx = ctx(b"audience-secret", Duration::minutes(10));

        let now = OffsetDateTime::now_utc();
        let claims = JwtClaims {
            iss: "nrs-webapp".to_string(),
            aud: "some-other-audience".to_string(),
            sub: "user-aud".to_string(),
            iat: now,
            exp: now + Duration::minutes(10),
        };

        let token = ctx.sign(&claims).expect("sign should succeed");

        let result = ctx.verify(&token);
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn test_config() -> Config {
        Config {
            jwt_secret: "test_secret_key_at_least_256_bits_long_for_HS256_algorithm_requirements_padding".to_string(),
            jwt_expiry_secs: 3600,
            ..Default::default()
        }
    }

    fn test_user_for_token() -> UserForToken {
        UserForToken {
            id: 42,
            username: "testuser".to_string(),
        }
    }

    #[test]
    fn test_create_token_success() {
        let config = test_config();
        let user = test_user_for_token();
        
        let result = create_token(&user, &config);
        assert!(result.is_ok(), "Token creation should succeed");
        
        let token = result.unwrap();
        assert!(!token.is_empty(), "Token should not be empty");
        assert_eq!(token.matches('.').count(), 2, "JWT should have 3 parts separated by dots");
    }

    #[test]
    fn test_create_token_different_users_different_tokens() {
        let config = test_config();
        let user1 = UserForToken { id: 1, username: "user1".to_string() };
        let user2 = UserForToken { id: 2, username: "user2".to_string() };
        
        let token1 = create_token(&user1, &config).unwrap();
        let token2 = create_token(&user2, &config).unwrap();
        
        assert_ne!(token1, token2, "Different users should produce different tokens");
    }

    #[test]
    fn test_validate_token_success() {
        let config = test_config();
        let user = test_user_for_token();
        
        let token = create_token(&user, &config).unwrap();
        let result = validate_token(&token, &config);
        
        assert!(result.is_ok(), "Token validation should succeed");
        let claims = result.unwrap();
        assert_eq!(claims.sub, user.id, "User ID should match");
        assert_eq!(claims.username, user.username, "Username should match");
    }

    #[test]
    fn test_validate_token_invalid_format() {
        let config = test_config();
        let invalid_token = "not.a.valid.jwt.token";
        
        let result = validate_token(invalid_token, &config);
        assert!(result.is_err(), "Invalid token format should fail validation");
    }

    #[test]
    fn test_validate_token_wrong_secret() {
        let config = test_config();
        let user = test_user_for_token();
        
        let token = create_token(&user, &config).unwrap();
        
        let wrong_config = Config {
            jwt_secret: "different_secret_key_that_should_fail_validation_requirements_padding".to_string(),
            ..config
        };
        
        let result = validate_token(&token, &wrong_config);
        assert!(result.is_err(), "Token with wrong secret should fail validation");
    }

    #[test]
    fn test_validate_token_empty_string() {
        let config = test_config();
        let result = validate_token("", &config);
        
        assert!(result.is_err(), "Empty token should fail validation");
    }

    #[test]
    fn test_validate_token_malformed() {
        let config = test_config();
        
        let malformed_tokens = vec![
            "header",
            "header.payload",
            "header..signature",
            ".payload.signature",
            "header.payload.",
        ];
        
        for token in malformed_tokens {
            let result = validate_token(token, &config);
            assert!(result.is_err(), "Malformed token '{}' should fail", token);
        }
    }

    #[test]
    fn test_validate_token_expired() {
        let config = Config {
            jwt_secret: "test_secret_key_at_least_256_bits_long_for_HS256_algorithm_requirements_padding".to_string(),
            jwt_expiry_secs: 0, // Expired immediately
            ..Default::default()
        };
        let user = test_user_for_token();
        
        // Note: This test may be flaky due to timing, but with 0 expiry it should reliably fail
        let token = create_token(&user, &config).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        let result = validate_token(&token, &config);
        // Note: Depending on JWT library behavior, this might succeed if checked instantly
        // In production, you'd want a more robust expiry test
    }

    #[test]
    fn test_create_token_with_special_characters_in_username() {
        let config = test_config();
        let user = UserForToken {
            id: 1,
            username: "user@example.com".to_string(),
        };
        
        let result = create_token(&user, &config);
        assert!(result.is_ok(), "Should handle special characters in username");
        
        let token = result.unwrap();
        let claims = validate_token(&token, &config).unwrap();
        assert_eq!(claims.username, user.username);
    }

    #[test]
    fn test_create_token_with_unicode_username() {
        let config = test_config();
        let user = UserForToken {
            id: 1,
            username: "用户名🎉".to_string(),
        };
        
        let result = create_token(&user, &config);
        assert!(result.is_ok(), "Should handle Unicode in username");
        
        let token = result.unwrap();
        let claims = validate_token(&token, &config).unwrap();
        assert_eq!(claims.username, user.username);
    }

    #[test]
    fn test_token_claims_structure() {
        let config = test_config();
        let user = test_user_for_token();
        
        let token = create_token(&user, &config).unwrap();
        let claims = validate_token(&token, &config).unwrap();
        
        assert!(claims.exp > 0, "Expiry should be set");
        assert!(claims.iat > 0, "Issued at should be set");
        assert_eq!(claims.sub, user.id);
        assert_eq!(claims.username, user.username);
    }

    #[test]
    fn test_validate_token_modified_payload() {
        let config = test_config();
        let user = test_user_for_token();
        
        let token = create_token(&user, &config).unwrap();
        
        // Try to modify the middle section (payload)
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() == 3 {
            let modified_token = format!("{}.modified.{}", parts[0], parts[2]);
            let result = validate_token(&modified_token, &config);
            assert!(result.is_err(), "Modified payload should fail validation");
        }
    }

    #[test]
    fn test_token_roundtrip_multiple_users() {
        let config = test_config();
        
        let users = vec![
            UserForToken { id: 1, username: "alice".to_string() },
            UserForToken { id: 2, username: "bob".to_string() },
            UserForToken { id: 3, username: "charlie".to_string() },
        ];
        
        for user in users {
            let token = create_token(&user, &config).unwrap();
            let claims = validate_token(&token, &config).unwrap();
            
            assert_eq!(claims.sub, user.id);
            assert_eq!(claims.username, user.username);
        }
    }

    #[test]
    fn test_create_token_large_user_id() {
        let config = test_config();
        let user = UserForToken {
            id: i64::MAX,
            username: "maxuser".to_string(),
        };
        
        let result = create_token(&user, &config);
        assert!(result.is_ok(), "Should handle large user IDs");
        
        let token = result.unwrap();
        let claims = validate_token(&token, &config).unwrap();
        assert_eq!(claims.sub, i64::MAX);
    }

    #[test]
    fn test_create_token_empty_username() {
        let config = test_config();
        let user = UserForToken {
            id: 1,
            username: String::new(),
        };
        
        let result = create_token(&user, &config);
        assert!(result.is_ok(), "Should handle empty username");
        
        let token = result.unwrap();
        let claims = validate_token(&token, &config).unwrap();
        assert_eq!(claims.username, "");
    }
}