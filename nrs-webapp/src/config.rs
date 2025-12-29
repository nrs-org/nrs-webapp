use std::{str::FromStr, sync::OnceLock, time::Duration};

use anyhow::Context;
use axum_client_ip::ClientIpSource;
use base64::{
    Engine as _,
    prelude::{BASE64_URL_SAFE, BASE64_URL_SAFE_NO_PAD},
};

#[derive(Debug)]
#[allow(non_snake_case)]
pub struct AppConfig {
    pub STATIC_SERVE_DIR: String,
    pub SERVICE_DB_URL: String,
    pub IP_SOURCE: ClientIpSource,

    pub SERVICE_PASSWORD_PEPPER: Vec<u8>,
    pub SERVICE_JWT_SECRET: Vec<u8>,
    pub SERVICE_JWT_EXPIRY_DURATION: Duration,

    pub SERVICE_TOKEN_SECRET: Vec<u8>,
    pub SERVICE_EMAIL_VERIFICATION_EXPIRY_DURATION: Duration,
    pub SERVICE_PASSWORD_RESET_EXPIRY_DURATION: Duration,
    pub RESEND_API_KEY: Option<String>,

    pub EMAIL_ACCOUNT_SUPPORT: Option<String>,
}

impl AppConfig {
    fn get_env(key: &'static str) -> anyhow::Result<String> {
        std::env::var(key).context(key)
    }

    fn get_env_parse<T: FromStr>(key: &'static str) -> anyhow::Result<T>
    where
        T::Err: std::error::Error + Send + Sync + 'static,
    {
        let value_str = Self::get_env(key)?;
        let value = value_str.parse::<T>()?;
        Ok(value)
    }

    fn get_env_dur_secs(key: &'static str) -> anyhow::Result<Duration> {
        let secs = Self::get_env_parse::<u64>(key)?;
        Ok(Duration::from_secs(secs))
    }

    fn get_env_b64u(key: &'static str) -> anyhow::Result<Vec<u8>> {
        let value_str = Self::get_env(key)?;
        let decoded = BASE64_URL_SAFE.decode(&value_str)?;
        Ok(decoded)
    }

    pub fn load_from_env() -> anyhow::Result<Self> {
        Ok(Self {
            STATIC_SERVE_DIR: Self::get_env("STATIC_SERVE_DIR")?,
            SERVICE_DB_URL: Self::get_env("SERVICE_DB_URL")?,
            IP_SOURCE: Self::get_env_parse::<ClientIpSource>("IP_SOURCE")?,
            SERVICE_PASSWORD_PEPPER: Self::get_env_b64u("SERVICE_PASSWORD_PEPPER")?,
            SERVICE_JWT_SECRET: Self::get_env_b64u("SERVICE_JWT_SECRET")?,
            SERVICE_JWT_EXPIRY_DURATION: Self::get_env_dur_secs("SERVICE_JWT_EXPIRY_SECS")?,
            SERVICE_EMAIL_VERIFICATION_EXPIRY_DURATION: Self::get_env_dur_secs(
                "SERVICE_EMAIL_VERIFICATION_EXPIRY_SECS",
            )?,
            SERVICE_PASSWORD_RESET_EXPIRY_DURATION: Self::get_env_dur_secs(
                "SERVICE_PASSWORD_RESET_EXPIRY_SECS",
            )?,
            RESEND_API_KEY: Self::get_env("RESEND_API_KEY").ok(),
            SERVICE_TOKEN_SECRET: Self::get_env_b64u("SERVICE_TOKEN_SECRET")?,
            EMAIL_ACCOUNT_SUPPORT: Self::get_env("EMAIL_ACCOUNT_SUPPORT").ok(),
        })
    }

    pub fn get() -> &'static Self {
        static INSTANCE: OnceLock<AppConfig> = OnceLock::new();
        INSTANCE.get_or_init(|| {
            Self::load_from_env().expect("Failed to load configuration from environment")
        })
    }

    fn duration_to_time_duration(dur: Duration) -> time::Duration {
        time::Duration::try_from(dur).expect("negative duration")
    }

    pub fn jwt_expiry_duration(&self) -> time::Duration {
        Self::duration_to_time_duration(self.SERVICE_JWT_EXPIRY_DURATION)
    }

    pub fn email_verification_expiry_duration(&self) -> time::Duration {
        Self::duration_to_time_duration(self.SERVICE_EMAIL_VERIFICATION_EXPIRY_DURATION)
    }

    pub fn password_reset_expiry_duration(&self) -> time::Duration {
        Self::duration_to_time_duration(self.SERVICE_PASSWORD_RESET_EXPIRY_DURATION)
    }
}
