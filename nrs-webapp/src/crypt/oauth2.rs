use super::Result;
use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use rand::{TryRngCore, rngs::OsRng};

pub const DEFAULT_STATE_LENGTH: usize = 32;

pub fn generate_state<const N: usize>() -> Result<String> {
    let mut bytes = [0u8; N];
    OsRng.try_fill_bytes(&mut bytes)?;
    Ok(BASE64_URL_SAFE_NO_PAD.encode(&bytes))
}

pub struct PkcePair {
    pub verifier: String,
    pub challenge: String,
    pub method: &'static str,
}

pub fn generate_pkce_verifier<const N: usize>() -> Result<String> {
    let mut bytes = [0u8; N];
    OsRng.try_fill_bytes(&mut bytes)?;
    Ok(BASE64_URL_SAFE_NO_PAD.encode(&bytes))
}

pub trait PkceMethod {
    fn generate_pair(verifier: &str) -> PkcePair {
        let challenge = Self::compute_challenge(verifier);
        PkcePair {
            verifier: verifier.to_string(),
            challenge,
            method: Self::name(),
        }
    }

    fn name() -> &'static str;
    fn compute_challenge(verifier: &str) -> String;
}

pub struct PkcePlain;

impl PkceMethod for PkcePlain {
    fn compute_challenge(verifier: &str) -> String {
        verifier.to_string()
    }

    fn name() -> &'static str {
        "plain"
    }
}

pub struct PkceSha256;

impl PkceMethod for PkceSha256 {
    fn compute_challenge(verifier: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(verifier.as_bytes());
        let hash = hasher.finalize();
        BASE64_URL_SAFE_NO_PAD.encode(hash)
    }

    fn name() -> &'static str {
        "S256"
    }
}

pub const DEFAULT_PKCE_VERIFIER_LENGTH: usize = 64;

pub fn generate_pkce_pair<const N: usize, M: PkceMethod>() -> Result<PkcePair> {
    let verifier = generate_pkce_verifier::<N>()?;
    Ok(M::generate_pair(&verifier))
}
