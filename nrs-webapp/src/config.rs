use std::{fs::File, str::FromStr, sync::OnceLock, time::Duration};

use anyhow::Context;
use axum_client_ip::ClientIpSource;
use base64::{Engine as _, prelude::BASE64_URL_SAFE};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GoogleOAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub auth_uri: String,
    pub token_uri: String,
}

#[derive(Debug)]
#[allow(non_snake_case)]
pub struct AppConfig {
    pub STATIC_SERVE_DIR: String,
    pub SERVICE_BASE_URL: String,
    pub SERVICE_DB_URL: String,
    pub IP_SOURCE: ClientIpSource,

    pub SERVICE_PASSWORD_PEPPER: Vec<u8>,
    pub SERVICE_COOKIE_KEY: Vec<u8>,
    pub SERVICE_SESSION_EXPIRY_DURATION: Duration,

    pub SERVICE_TOKEN_SECRET: Vec<u8>,
    pub SERVICE_EMAIL_VERIFICATION_EXPIRY_DURATION: Duration,
    pub SERVICE_PASSWORD_RESET_EXPIRY_DURATION: Duration,
    pub RESEND_API_KEY: Option<String>,

    pub EMAIL_ACCOUNT_SUPPORT: Option<String>,
    pub GOOGLE_OAUTH_CREDENTIALS: Option<GoogleOAuthConfig>,
}

impl AppConfig {
    /// Fetches the value of the environment variable named by `key`, returning an error that is annotated with the key if retrieval fails.
    ///
    /// On success, returns the environment variable's string value. On failure (for example if the variable is not set), the returned `anyhow::Error` is given context containing the provided `key`.
    ///
    /// # Examples
    ///
    /// ```
    /// std::env::set_var("MY_APP_KEY", "value123");
    /// let val = crate::config::get_env("MY_APP_KEY").unwrap();
    /// assert_eq!(val, "value123");
    /// ```
    fn get_env(key: &'static str) -> anyhow::Result<String> {
        std::env::var(key).context(key)
    }

    /// Fetches the environment variable named `key` and parses its value into `T`.
    ///
    /// Returns the parsed value of type `T`.
    ///
    /// # Errors
    /// Returns an error if the environment variable is missing or if parsing into `T` fails.
    ///
    /// # Examples
    ///
    /// ```
    /// // Assuming the environment variable `MAX_RETRIES="5"` is set:
    /// let n: u32 = AppConfig::get_env_parse("MAX_RETRIES").unwrap();
    /// assert_eq!(n, 5);
    /// ```
    fn get_env_parse<T: FromStr>(key: &'static str) -> anyhow::Result<T>
    where
        T::Err: std::error::Error + Send + Sync + 'static,
    {
        let value_str = Self::get_env(key)?;
        let value = value_str.parse::<T>()?;
        Ok(value)
    }

    /// Convert an environment variable containing a number of seconds into a `std::time::Duration`.
    ///
    /// The environment variable named by `key` is parsed as an unsigned 64-bit integer and converted
    /// to a `Duration` representing that many seconds.
    ///
    /// # Errors
    ///
    /// Returns an error if the environment variable is missing or cannot be parsed as a `u64`.
    ///
    /// # Examples
    ///
    /// ```
    /// std::env::set_var("TEST_SECS", "3");
    /// let dur = crate::config::AppConfig::get_env_dur_secs("TEST_SECS").unwrap();
    /// assert_eq!(dur.as_secs(), 3);
    /// ```
    fn get_env_dur_secs(key: &'static str) -> anyhow::Result<Duration> {
        let secs = Self::get_env_parse::<u64>(key)?;
        Ok(Duration::from_secs(secs))
    }

    /// Decode a URL-safe base64-encoded environment variable into raw bytes.
    ///
    /// # Parameters
    ///
    /// - `key`: The name of the environment variable containing URL-safe base64 data.
    ///
    /// # Returns
    ///
    /// A `Vec<u8>` with the decoded bytes; returns an error if the environment variable is missing or contains invalid base64.
    ///
    /// # Examples
    ///
    /// ```
    /// std::env::set_var("TEST_SECRET_B64U", "c2VjcmV0"); // "secret" in base64 (URL-safe form is identical here)
    /// let bytes = crate::config::AppConfig::get_env_b64u("TEST_SECRET_B64U").unwrap();
    /// assert_eq!(bytes, b"secret");
    /// ```
    fn get_env_b64u(key: &'static str) -> anyhow::Result<Vec<u8>> {
        let value_str = Self::get_env(key)?;
        let decoded = BASE64_URL_SAFE
            .decode(&value_str)
            .with_context(|| format!("Invalid base64 for key {}", key))?;
        Ok(decoded)
    }

    /// Load an AppConfig by reading required and optional values from environment variables.
    ///
    /// Required environment variables:
    /// - `STATIC_SERVE_DIR`, `SERVICE_DB_URL` (strings)
    /// - `IP_SOURCE` (parsed as `ClientIpSource`)
    /// - `SERVICE_PASSWORD_PEPPER`, `SERVICE_COOKIE_KEY`, `SERVICE_TOKEN_SECRET` (URL-safe base64 decoded to `Vec<u8>`)
    /// - `SERVICE_SESSION_EXPIRY_SECS`, `SERVICE_EMAIL_VERIFICATION_EXPIRY_SECS`, `SERVICE_PASSWORD_RESET_EXPIRY_SECS` (seconds parsed to `std::time::Duration`)
    ///
    /// Optional environment variables (treated as `Option<String>`):
    /// - `RESEND_API_KEY`, `EMAIL_ACCOUNT_SUPPORT`
    ///
    /// # Errors
    ///
    /// Returns an error if any required environment variable is missing or cannot be parsed/decoded.
    ///
    /// # Examples
    ///
    /// ```
    /// // At runtime ensure required environment variables are set.
    /// let cfg = nrs_webapp::config::AppConfig::load_from_env().unwrap();
    /// assert!(cfg.SERVICE_DB_URL.len() > 0);
    /// ```
    pub fn load_from_env() -> anyhow::Result<Self> {
        Ok(Self {
            STATIC_SERVE_DIR: Self::get_env("STATIC_SERVE_DIR")?,
            SERVICE_BASE_URL: Self::get_env("SERVICE_BASE_URL")?,
            SERVICE_DB_URL: Self::get_env("SERVICE_DB_URL")?,
            IP_SOURCE: Self::get_env_parse::<ClientIpSource>("IP_SOURCE")?,
            SERVICE_PASSWORD_PEPPER: Self::get_env_b64u("SERVICE_PASSWORD_PEPPER")?,
            SERVICE_COOKIE_KEY: Self::get_env_b64u("SERVICE_COOKIE_KEY")?,
            SERVICE_SESSION_EXPIRY_DURATION: Self::get_env_dur_secs("SERVICE_SESSION_EXPIRY_SECS")?,
            SERVICE_EMAIL_VERIFICATION_EXPIRY_DURATION: Self::get_env_dur_secs(
                "SERVICE_EMAIL_VERIFICATION_EXPIRY_SECS",
            )?,
            SERVICE_PASSWORD_RESET_EXPIRY_DURATION: Self::get_env_dur_secs(
                "SERVICE_PASSWORD_RESET_EXPIRY_SECS",
            )?,
            RESEND_API_KEY: Self::get_env("RESEND_API_KEY").ok(),
            SERVICE_TOKEN_SECRET: Self::get_env_b64u("SERVICE_TOKEN_SECRET")?,
            EMAIL_ACCOUNT_SUPPORT: Self::get_env("EMAIL_ACCOUNT_SUPPORT").ok(),
            GOOGLE_OAUTH_CREDENTIALS: Self::load_google_oauth_config()
                .inspect_err(|err| tracing::warn!("GOOGLE_OAUTH_CREDENTIALS not loaded: {err:?}"))
                .ok(),
        })
    }

    /// Returns a global, lazily-initialized singleton AppConfig instance.
    ///
    /// On first call the configuration is loaded from the process environment; subsequent calls return
    /// the same static reference. Panics if loading the configuration fails.
    ///
    /// # Returns
    ///
    /// A `'static` reference to the initialized `AppConfig`.
    ///
    /// # Examples
    ///
    /// ```
    /// let cfg: &AppConfig = AppConfig::get();
    /// // use `cfg` (e.g., read fields)...
    /// assert!(!std::ptr::eq(cfg, std::ptr::null()));
    /// ```
    pub fn get() -> &'static Self {
        static INSTANCE: OnceLock<AppConfig> = OnceLock::new();
        INSTANCE.get_or_init(|| {
            Self::load_from_env().expect("Failed to load configuration from environment")
        })
    }

    /// Convert a `std::time::Duration` into a `time::Duration`.
    ///
    /// # Panics
    ///
    /// Panics if `dur` cannot be converted (for example, if it would represent a negative `time::Duration`).
    ///
    /// # Examples
    ///
    /// ```
    /// let s = std::time::Duration::from_secs(5);
    /// let t = duration_to_time_duration(s);
    /// assert_eq!(t.whole_seconds(), 5);
    /// ```
    fn duration_to_time_duration(dur: Duration) -> time::Duration {
        time::Duration::try_from(dur).expect("negative duration")
    }

    /// Get the configured session expiry as a `time::Duration`.
    ///
    /// Converts the stored `SERVICE_SESSION_EXPIRY_DURATION` (an `std::time::Duration`) into a `time::Duration`.
    ///
    /// # Examples
    ///
    /// ```
    /// let expiry = AppConfig::get().session_expiry_duration();
    /// // `expiry` is a `time::Duration`
    /// ```
    pub fn session_expiry_duration(&self) -> time::Duration {
        Self::duration_to_time_duration(self.SERVICE_SESSION_EXPIRY_DURATION)
    }

    /// Provides the configured email verification expiry as a `time::Duration`.
    ///
    /// The value is derived from the `SERVICE_EMAIL_VERIFICATION_EXPIRY_DURATION` configuration (stored as
    /// `std::time::Duration`) and converted to `time::Duration`.
    ///
    /// # Examples
    ///
    /// ```
    /// let dur = AppConfig::get().email_verification_expiry_duration();
    /// assert!(dur > time::Duration::seconds(0));
    /// ```
    pub fn email_verification_expiry_duration(&self) -> time::Duration {
        Self::duration_to_time_duration(self.SERVICE_EMAIL_VERIFICATION_EXPIRY_DURATION)
    }

    /// Returns the password-reset expiry as a `time::Duration`.
    ///
    /// # Returns
    ///
    /// `time::Duration` representing how long a password reset token remains valid.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let expiry = AppConfig::get().password_reset_expiry_duration();
    /// println!("Password reset tokens expire after {} seconds", expiry.whole_seconds());
    /// ```
    pub fn password_reset_expiry_duration(&self) -> time::Duration {
        Self::duration_to_time_duration(self.SERVICE_PASSWORD_RESET_EXPIRY_DURATION)
    }

    fn load_google_oauth_config() -> anyhow::Result<GoogleOAuthConfig> {
        #[derive(Deserialize)]
        struct GoogleOAuthConfigWrapped {
            web: GoogleOAuthConfig,
        }

        let path = Self::get_env("GOOGLE_OAUTH_CREDENTIALS_PATH")?;
        tracing::info!("Loading Google OAuth credentials from {}", path);
        let credentials =
            serde_json::from_reader::<_, GoogleOAuthConfigWrapped>(File::open(&path)?)
                .with_context(|| {
                    format!("Failed to read Google OAuth credentials from {}", path)
                })?;

        Ok(credentials.web)
    }
}
