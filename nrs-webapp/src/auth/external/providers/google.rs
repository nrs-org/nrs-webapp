use crate::auth::external::auth_url::{
    AuthorizeUrl, BaseAuthorizeUrlGenerator, OidcAuthorizeUrlGeneratorTrait,
};
use crate::auth::external::exch_code::{BaseCodeExchanger, OidcCodeExchangerTrait};
use crate::auth::external::oidc_discover::oidc_discover;
use crate::auth::external::oidc_fetch_identity::{BaseIdentityFetcher, OidcIdentityFetcherTrait};
use crate::auth::external::{AuthProvider, IdToken, TokenResponse, UserIdentity};
use crate::model::ModelManager;
use async_trait::async_trait;
use oauth2::basic::{BasicErrorResponseType, BasicTokenType};
use oauth2::{
    AccessToken, EmptyExtraTokenFields, EndpointMaybeSet, EndpointNotSet, EndpointSet,
    PkceCodeVerifier, RevocationErrorResponseType, StandardErrorResponse, StandardRevocableToken,
    StandardTokenIntrospectionResponse, StandardTokenResponse,
};
use openidconnect::DiscoveryError;
use openidconnect::core::{
    CoreAuthDisplay, CoreAuthPrompt, CoreClaimName, CoreClaimType, CoreClient,
    CoreClientAuthMethod, CoreGenderClaim, CoreGrantType, CoreJsonWebKey,
    CoreJweContentEncryptionAlgorithm, CoreJweKeyManagementAlgorithm, CoreJwsSigningAlgorithm,
    CoreResponseMode, CoreResponseType, CoreSubjectIdentifierType,
};
use openidconnect::{
    AdditionalProviderMetadata, Client, ClientId, ClientSecret, EmptyAdditionalClaims,
    IdTokenFields, Nonce, ProviderMetadata, RedirectUrl, RevocationUrl,
};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::auth::Result;

#[derive(Clone, Debug, Deserialize, Serialize)]
struct RevocationEndpointProviderMetadata {
    revocation_endpoint: String,
}
impl AdditionalProviderMetadata for RevocationEndpointProviderMetadata {}

type GoogleProviderMetadata = ProviderMetadata<
    RevocationEndpointProviderMetadata,
    CoreAuthDisplay,
    CoreClientAuthMethod,
    CoreClaimName,
    CoreClaimType,
    CoreGrantType,
    CoreJweContentEncryptionAlgorithm,
    CoreJweKeyManagementAlgorithm,
    CoreJsonWebKey,
    CoreResponseMode,
    CoreResponseType,
    CoreSubjectIdentifierType,
>;

type GoogleCoreClient = Client<
    EmptyAdditionalClaims,
    CoreAuthDisplay,
    CoreGenderClaim,
    CoreJweContentEncryptionAlgorithm,
    CoreJsonWebKey,
    CoreAuthPrompt,
    StandardErrorResponse<BasicErrorResponseType>,
    StandardTokenResponse<
        IdTokenFields<
            EmptyAdditionalClaims,
            EmptyExtraTokenFields,
            CoreGenderClaim,
            CoreJweContentEncryptionAlgorithm,
            CoreJwsSigningAlgorithm,
        >,
        BasicTokenType,
    >,
    StandardTokenIntrospectionResponse<EmptyExtraTokenFields, BasicTokenType>,
    StandardRevocableToken,
    StandardErrorResponse<RevocationErrorResponseType>,
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointSet,
    EndpointMaybeSet,
    EndpointMaybeSet,
>;

struct GoogleCoreClientWrapper(GoogleCoreClient);

impl BaseAuthorizeUrlGenerator for GoogleCoreClientWrapper {
    type Client = GoogleCoreClient;

    fn as_client(&self) -> &Self::Client {
        &self.0
    }

    fn scopes(&self) -> &'static [&'static str] {
        &["email", "profile"]
    }
}

impl BaseCodeExchanger for GoogleCoreClientWrapper {
    type Client = GoogleCoreClient;

    fn as_client(&self) -> &Self::Client {
        &self.0
    }
}

impl BaseIdentityFetcher for GoogleCoreClientWrapper {
    type Client = GoogleCoreClient;

    fn as_client(&self) -> &Self::Client {
        &self.0
    }
}

pub struct GoogleAuthProvider {
    client_id: String,
    client_secret: String,
}

impl GoogleAuthProvider {
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            client_id,
            client_secret,
        }
    }

    pub fn from_config() -> Option<Self> {
        let config = crate::config::AppConfig::get()
            .GOOGLE_OAUTH_CREDENTIALS
            .as_ref()?;
        Some(Self::new(
            config.client_id.clone(),
            config.client_secret.clone(),
        ))
    }

    async fn client(
        &self,
        mm: &ModelManager,
        redirect_uri: Url,
    ) -> Result<GoogleCoreClientWrapper> {
        let provider_metadata: GoogleProviderMetadata =
            oidc_discover(mm, "https://accounts.google.com").await?;
        let revocation_endpoint = provider_metadata
            .additional_metadata()
            .revocation_endpoint
            .clone();
        let client = CoreClient::from_provider_metadata(
            provider_metadata,
            ClientId::new(self.client_id.clone()),
            Some(ClientSecret::new(self.client_secret.clone())),
        )
        .set_redirect_uri(RedirectUrl::from_url(redirect_uri))
        .set_revocation_url(
            RevocationUrl::new(revocation_endpoint).map_err(DiscoveryError::UrlParse)?,
        );
        Ok(GoogleCoreClientWrapper(client))
    }
}

#[async_trait]
impl AuthProvider for GoogleAuthProvider {
    fn name(&self) -> &'static str {
        "google"
    }

    async fn authorize_url(&self, mm: &ModelManager, redirect_uri: Url) -> Result<AuthorizeUrl> {
        self.client(mm, redirect_uri)
            .await?
            .create_authorize_url_oidc()
    }

    async fn exchange_code(
        &self,
        mm: &ModelManager,
        code: String,
        redirect_uri: Url,
        pkce_verifier: Option<PkceCodeVerifier>,
    ) -> Result<(TokenResponse, IdToken)> {
        self.client(mm, redirect_uri)
            .await?
            .exchange_code_oidc(mm.http_client_wrapper(), code, pkce_verifier)
            .await
    }

    async fn fetch_identity(
        &self,
        mm: &ModelManager,
        id_token: IdToken,
        nonce: Option<Nonce>,
        access_token: &AccessToken,
        redirect_uri: Url,
    ) -> Result<UserIdentity> {
        self.client(mm, redirect_uri)
            .await?
            .fetch_identity_oidc(mm.http_client_wrapper(), &id_token, access_token, nonce)
            .await
    }
}
