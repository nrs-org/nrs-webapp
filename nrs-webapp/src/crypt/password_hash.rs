use std::sync::OnceLock;

use argon2::{
    Algorithm, Argon2, PasswordHasher as _, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};

use super::Result;
use crate::config::AppConfig;

pub struct PasswordHasher(Argon2<'static>);

impl PasswordHasher {
    /// Create a PasswordHasher configured with the provided secret pepper.
    ///
    /// `pepper` is used as the Argon2 secret (a global, static secret added to every hash).
    ///
    /// # Examples
    ///
    /// ```
    /// let hasher = PasswordHasher::new(b"my-secret-pepper").expect("create hasher");
    /// ```
    pub fn new(pepper: &'static [u8]) -> argon2::Result<Self> {
        Ok(PasswordHasher(Argon2::new_with_secret(
            pepper,
            Default::default(),
            Default::default(),
            Default::default(),
        )?))
    }

    /// Returns a lazily-initialized static PasswordHasher configured from application config.
    ///
    /// The hasher is created once on first call using AppConfig::get().SERVICE_PASSWORD_PEPPER and reused for the lifetime of the program. Initialization will panic if the underlying Argon2 construction fails.
    ///
    /// # Examples
    ///
    /// ```
    /// let hasher = PasswordHasher::get_from_config();
    /// let _ref: &PasswordHasher = hasher;
    /// ```
    pub fn get_from_config() -> &'static Self {
        static HASHER: OnceLock<PasswordHasher> = OnceLock::new();
        HASHER.get_or_init(|| {
            PasswordHasher::new(&AppConfig::get().SERVICE_PASSWORD_PEPPER)
                .expect("Failed to create PasswordHasher")
        })
    }

    /// Hashes the given password with a newly generated random salt and returns the PHC-encoded Argon2 hash string.
    ///
    /// The resulting string contains the algorithm, parameters, salt, and hash in PHC format.
    ///
    /// # Examples
    ///
    /// ```
    /// let hasher = PasswordHasher::new(b"test-pepper").unwrap();
    /// let hash = hasher.encrypt_password("secret").unwrap();
    /// assert!(hash.starts_with("$argon2"));
    /// ```
    ///
    /// # Returns
    ///
    /// `Ok` with the PHC-encoded password hash string on success, or an error from the underlying Argon2 hashing operation.
    pub fn encrypt_password(&self, password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = self
            .0
            .hash_password(password.as_bytes(), &salt)?
            .to_string();
        Ok(password_hash)
    }

    /// Verifies whether a plaintext password matches an Argon2-encoded password hash.
    ///
    /// Parses the provided hash string and compares it against the supplied plaintext password
    /// using the hasher's configured secret (pepper). Returns `true` when the password matches,
    /// `false` when it does not, and an error if the hash is malformed or another verification
    /// error occurs.
    ///
    /// # Parameters
    ///
    /// - `password_hash`: An Argon2-encoded password hash string produced by `encrypt_password`.
    ///
    /// # Returns
    ///
    /// `true` if the password matches the provided hash, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nrs_webapp::crypt::password_hash::PasswordHasher;
    /// let hasher = PasswordHasher::new(b"example-pepper").unwrap();
    /// let hash = hasher.encrypt_password("s3cr3t").unwrap();
    /// assert!(hasher.verify_password("s3cr3t", &hash).unwrap());
    /// ```
    pub fn verify_password(&self, password: &str, password_hash: &str) -> Result<bool> {
        let parsed_hash = argon2::PasswordHash::new(password_hash)?;
        match self.0.verify_password(password.as_bytes(), &parsed_hash) {
            Ok(_) => Ok(true),
            Err(argon2::password_hash::Error::Password) => Ok(false),
            Err(e) => Err(e.into()),
        }
    }

    /// Provides a static, memoized dummy password hash for use in tests or fallbacks.
    ///
    /// The value is computed once using the global `PasswordHasher` and cached for the program's lifetime.
    ///
    /// # Examples
    ///
    /// ```
    /// let h = PasswordHasher::get_from_config().dummy_hash();
    /// assert!(!h.is_empty());
    /// ```
    pub fn dummy_hash(&self) -> &'static str {
        static HASH: OnceLock<String> = OnceLock::new();
        HASH.get_or_init(|| {
            PasswordHasher::get_from_config()
                .encrypt_password("tententengokujigokugoku")
                .expect("Failed to create dummy hash")
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hasher() -> PasswordHasher {
        PasswordHasher::new(b"test-pepper").expect("failed to create PasswordHasher")
    }

    #[test]
    fn encrypt_and_verify_password_success() {
        let hasher = hasher();
        let password = "correct horse battery staple";

        let hash = hasher
            .encrypt_password(password)
            .expect("hashing should succeed");

        let ok = hasher
            .verify_password(password, &hash)
            .expect("verification should succeed");

        assert!(ok);
    }

    #[test]
    fn verify_password_fails_for_wrong_password() {
        let hasher = hasher();

        let hash = hasher
            .encrypt_password("right-password")
            .expect("hashing should succeed");

        let ok = hasher
            .verify_password("wrong-password", &hash)
            .expect("verification should not error");

        assert!(!ok);
    }

    #[test]
    fn same_password_produces_different_hashes_due_to_salt() {
        let hasher = hasher();
        let password = "same-password";

        let hash1 = hasher.encrypt_password(password).unwrap();
        let hash2 = hasher.encrypt_password(password).unwrap();

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn verify_rejects_malformed_hash() {
        let hasher = hasher();

        let result = hasher.verify_password("password", "not-a-valid-hash");

        assert!(result.is_err());
    }

    #[test]
    fn verify_fails_if_pepper_is_different() {
        let hasher_good = PasswordHasher::new(b"pepper-one").unwrap();
        let hasher_bad = PasswordHasher::new(b"pepper-two").unwrap();

        let hash = hasher_good
            .encrypt_password("password")
            .expect("hashing should succeed");

        let ok = hasher_bad
            .verify_password("password", &hash)
            .expect("verification should not error");

        assert!(!ok);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password_success() {
        let password = "MySecureP@ssw0rd123";
        let result = hash_password(password);
        
        assert!(result.is_ok(), "Password hashing should succeed");
        let hash = result.unwrap();
        assert!(!hash.is_empty(), "Hash should not be empty");
        assert!(hash.starts_with("$argon2"), "Hash should be Argon2 format");
    }

    #[test]
    fn test_hash_password_empty_string() {
        let password = "";
        let result = hash_password(password);
        
        assert!(result.is_ok(), "Should handle empty password");
    }

    #[test]
    fn test_hash_password_long_password() {
        let password = "a".repeat(1000);
        let result = hash_password(&password);
        
        assert!(result.is_ok(), "Should handle long passwords");
    }

    #[test]
    fn test_hash_password_special_characters() {
        let password = "P@$$w0rd!#%&*()[]{}|;:,.<>?/~`";
        let result = hash_password(password);
        
        assert!(result.is_ok(), "Should handle special characters");
    }

    #[test]
    fn test_hash_password_unicode() {
        let password = "пароль密码🔐";
        let result = hash_password(password);
        
        assert!(result.is_ok(), "Should handle Unicode characters");
    }

    #[test]
    fn test_verify_password_correct() {
        let password = "TestPassword123!";
        let hash = hash_password(password).unwrap();
        
        let result = verify_password(password, &hash);
        assert!(result.is_ok(), "Verification should succeed");
        assert!(result.unwrap(), "Password should match hash");
    }

    #[test]
    fn test_verify_password_incorrect() {
        let password = "TestPassword123!";
        let wrong_password = "WrongPassword456!";
        let hash = hash_password(password).unwrap();
        
        let result = verify_password(wrong_password, &hash);
        assert!(result.is_ok(), "Verification should not error");
        assert!(!result.unwrap(), "Wrong password should not match");
    }

    #[test]
    fn test_verify_password_case_sensitive() {
        let password = "TestPassword";
        let hash = hash_password(password).unwrap();
        
        let result = verify_password("testpassword", &hash);
        assert!(result.is_ok());
        assert!(!result.unwrap(), "Password verification should be case-sensitive");
    }

    #[test]
    fn test_verify_password_invalid_hash_format() {
        let password = "TestPassword123!";
        let invalid_hash = "not_a_valid_argon2_hash";
        
        let result = verify_password(password, invalid_hash);
        assert!(result.is_err(), "Should fail with invalid hash format");
    }

    #[test]
    fn test_verify_password_empty_password_against_hash() {
        let password = "";
        let hash = hash_password(password).unwrap();
        
        let result = verify_password("", &hash);
        assert!(result.is_ok());
        assert!(result.unwrap(), "Empty password should verify against its hash");
        
        let result_wrong = verify_password("notempty", &hash);
        assert!(result_wrong.is_ok());
        assert!(!result_wrong.unwrap(), "Non-empty should not match empty hash");
    }

    #[test]
    fn test_hash_password_deterministic() {
        let password = "SamePassword123";
        let hash1 = hash_password(password).unwrap();
        let hash2 = hash_password(password).unwrap();
        
        // Hashes should be different due to random salt
        assert_ne!(hash1, hash2, "Same password should produce different hashes (salted)");
        
        // But both should verify correctly
        assert!(verify_password(password, &hash1).unwrap());
        assert!(verify_password(password, &hash2).unwrap());
    }

    #[test]
    fn test_verify_password_whitespace_differences() {
        let password = "password";
        let hash = hash_password(password).unwrap();
        
        assert!(!verify_password("password ", &hash).unwrap(), "Trailing space should not match");
        assert!(!verify_password(" password", &hash).unwrap(), "Leading space should not match");
        assert!(!verify_password("pass word", &hash).unwrap(), "Internal space should not match");
    }

    #[test]
    fn test_hash_password_boundary_lengths() {
        // Test very short password
        let short = "a";
        assert!(hash_password(short).is_ok());
        
        // Test reasonable max length (most systems limit to 72-128 chars for bcrypt/argon2)
        let long = "a".repeat(200);
        assert!(hash_password(&long).is_ok());
    }

    #[test]
    fn test_verify_password_hash_truncation() {
        let password = "TestPassword";
        let hash = hash_password(password).unwrap();
        let truncated = &hash[..hash.len() - 5];
        
        let result = verify_password(password, truncated);
        assert!(result.is_err(), "Truncated hash should fail verification");
    }

    #[test]
    fn test_verify_password_modified_hash() {
        let password = "TestPassword";
        let mut hash = hash_password(password).unwrap();
        
        // Modify a character in the middle
        if let Some(ch) = hash.chars().nth(20) {
            let replacement = if ch == 'a' { 'b' } else { 'a' };
            hash.replace_range(20..21, &replacement.to_string());
        }
        
        let result = verify_password(password, &hash);
        assert!(result.is_err() || !result.unwrap(), "Modified hash should not verify");
    }
}