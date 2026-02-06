use std::sync::OnceLock;

use crate::config::AppConfig;

use super::{Error, Result};
use aes_gcm::{
    AeadCore, Aes256Gcm, KeyInit, Nonce,
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
        let mut ciphertext = self.cipher.encrypt(&nonce, plaintext)?;
        ciphertext.extend_from_slice(&nonce);
        Ok(ciphertext)
    }

    pub fn decrypt(&self, ciphertext_with_nonce: &[u8]) -> Result<Vec<u8>> {
        const NONCE_SIZE: usize = std::mem::size_of::<Nonce<<Aes256Gcm as AeadCore>::NonceSize>>();
        if ciphertext_with_nonce.len() < NONCE_SIZE {
            return Err(Error::CiphertextTooShort);
        }
        let (ciphertext, nonce_bytes) =
            ciphertext_with_nonce.split_at(ciphertext_with_nonce.len() - NONCE_SIZE);
        let nonce = aes_gcm::Nonce::from_slice(nonce_bytes);
        let plaintext = self.cipher.decrypt(nonce, ciphertext)?;
        Ok(plaintext)
    }
}
