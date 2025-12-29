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