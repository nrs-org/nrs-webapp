use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use oauth2::{AccessToken, PkceCodeVerifier};
use openidconnect::Nonce;
use url::Url;

use crate::{
    auth::external::{
        auth_url::AuthorizeUrl,
        exch_code::{IdToken, TokenResponse},
    },
    model::ModelManager,
};

pub mod auth_url;
mod error;
pub mod exch_code;
pub mod oidc_discover;
pub mod oidc_fetch_identity;
mod providers;

pub use error::{Error, Result};

#[derive(Debug, Default, Clone)]
pub struct UserIdentity {
    pub id: String,
    pub username: Option<String>,
    pub email: Option<String>,
    pub email_verified: bool,
    pub profile_picture: Option<Url>,
}

#[async_trait]
pub trait AuthProvider: Send + Sync {
    fn name(&self) -> &'static str;

    async fn authorize_url(&self, mm: &ModelManager, redirect_uri: Url) -> Result<AuthorizeUrl>;

    async fn exchange_code(
        &self,
        mm: &ModelManager,
        code: String,
        redirect_uri: Url,
        pkce_verifier: Option<PkceCodeVerifier>,
    ) -> Result<(TokenResponse, IdToken)>;

    async fn fetch_identity(
        &self,
        mm: &ModelManager,
        id_token: IdToken,
        nonce: Option<Nonce>,
        access_token: &AccessToken,
        redirect_uri: Url,
    ) -> Result<UserIdentity>;
}

#[derive(Default, Clone)]
pub struct AuthProviderRegistry(Arc<HashMap<&'static str, Box<dyn AuthProvider>>>);

impl AuthProviderRegistry {
    pub fn new() -> Self {
        Self(Arc::new(HashMap::new()))
    }

    pub fn from_config() -> Self {
        let mut registry: HashMap<&'static str, Box<dyn AuthProvider>> = HashMap::new();

        for p in [providers::google(), providers::github()]
            .into_iter()
            .flatten()
        {
            registry.insert(p.name(), p);
        }

        Self(Arc::new(registry))
    }

    pub fn get(&self, name: &str) -> Option<&dyn AuthProvider> {
        self.0.get(name).map(|b| b.as_ref())
    }

    pub fn has_provider(&self, name: &str) -> bool {
        self.0.contains_key(name)
    }
}
