use std::{any::Any, collections::HashMap, sync::Arc};

use super::Result;
use async_trait::async_trait;
use oauth2::{AccessToken, CsrfToken, PkceCodeVerifier, RefreshToken};
use openidconnect::Nonce;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use url::Url;

use crate::model::ModelManager;

mod providers;

#[derive(Serialize, Deserialize)]
pub struct AuthFlowState {
    pub csrf_state: Option<CsrfToken>,
    pub nonce: Option<Nonce>,
    pub pkce_verifier: Option<PkceCodeVerifier>,
}

pub struct AuthorizeUrl {
    pub url: Url,
    pub state: AuthFlowState,
}

pub struct IdToken(pub(super) Box<dyn Any + Send>);

#[derive(Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: AccessToken,
    pub refresh_token: Option<RefreshToken>,
    pub expires_at: Option<OffsetDateTime>,
}

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

        for p in [providers::google()].into_iter().flatten() {
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
