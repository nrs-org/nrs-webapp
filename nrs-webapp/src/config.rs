use std::{str::FromStr, sync::OnceLock};

use base64::{
    Engine as _,
    prelude::{BASE64_URL_SAFE, BASE64_URL_SAFE_NO_PAD},
};

#[derive(Debug)]
#[allow(non_snake_case)]
pub struct AppConfig {
    pub STATIC_SERVE_DIR: String,
    pub SERVICE_DB_URL: String,
}

impl AppConfig {
    fn get_env(key: &str) -> anyhow::Result<String> {
        Ok(std::env::var(key)?)
    }

    fn get_env_parse<T: FromStr>(key: &str) -> anyhow::Result<T>
    where
        T::Err: std::error::Error + Send + Sync + 'static,
    {
        let value_str = Self::get_env(key)?;
        let value = value_str.parse::<T>()?;
        Ok(value)
    }

    fn get_env_b64u(key: &str) -> anyhow::Result<Vec<u8>> {
        let value_str = Self::get_env(key)?;
        let decoded = BASE64_URL_SAFE.decode(&value_str)?;
        Ok(decoded)
    }

    pub fn load_from_env() -> anyhow::Result<Self> {
        Ok(Self {
            STATIC_SERVE_DIR: Self::get_env("STATIC_SERVE_DIR")?,
            SERVICE_DB_URL: Self::get_env("SERVICE_DB_URL")?,
        })
    }

    pub fn get() -> &'static Self {
        static INSTANCE: OnceLock<AppConfig> = OnceLock::new();
        INSTANCE.get_or_init(|| {
            Self::load_from_env().expect("Failed to load configuration from environment")
        })
    }
}
