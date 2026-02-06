use std::sync::OnceLock;

use crate::config::AppConfig;

use super::Result;
use aes_gcm::{
    AeadCore, Aes256Gcm, KeyInit,
    aead::{Aead, OsRng},
};

pub struct SymmetricCipher {
    cipher: Aes256Gcm,
}

impl SymmetricCipher {
    pub fn new(key: &[u8]) -> core::result::Result<Self, anyhow::Error> {
        Ok(Self {
            cipher: Aes256Gcm::new_from_slice(key)?,
        })
    }

    pub fn get_from_config() -> &'static Self {
        static INSTANCE: OnceLock<SymmetricCipher> = OnceLock::new();
        // nrs-keygen currently generates fixed-length 128-byte keys, so to avoid the
        // InvalidLength error we only use the first 32 bytes.
        // TODO: address this
        INSTANCE.get_or_init(|| {
            SymmetricCipher::new(&AppConfig::get().SERVICE_ENCRYPTION_KEY[0..32])
                .expect("invalid symmetric encryption key")
        })
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        Ok(self.cipher.encrypt(&nonce, plaintext)?)
    }
}
