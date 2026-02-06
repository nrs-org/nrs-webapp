use std::borrow::Cow;

use crate::auth::external::{
    AuthFlowState, AuthProvider, AuthorizeUrl, IdToken, TokenResponse, UserIdentity,
};
use crate::model::ModelManager;
use async_trait::async_trait;
use oauth2::basic::{BasicErrorResponseType, BasicTokenType};
use oauth2::{
    AuthorizationCode, EmptyExtraTokenFields, EndpointMaybeSet, EndpointNotSet, EndpointSet,
    PkceCodeChallenge, PkceCodeVerifier, RevocationErrorResponseType, StandardErrorResponse,
    StandardRevocableToken, StandardTokenIntrospectionResponse, StandardTokenResponse,
    TokenResponse as _,
};
use openidconnect::core::{
    CoreAuthDisplay, CoreAuthPrompt, CoreClaimName, CoreClaimType, CoreClient,
    CoreClientAuthMethod, CoreGenderClaim, CoreGrantType, CoreJsonWebKey,
    CoreJweContentEncryptionAlgorithm, CoreJweKeyManagementAlgorithm, CoreJwsSigningAlgorithm,
    CoreResponseMode, CoreResponseType, CoreSubjectIdentifierType,
};
use openidconnect::{
    AdditionalProviderMetadata, AuthenticationFlow, Client, ClientId, ClientSecret, CsrfToken,
    EmptyAdditionalClaims, IdTokenFields, IssuerUrl, Nonce, ProviderMetadata, RedirectUrl,
    RevocationUrl, Scope,
};
use openidconnect::{DiscoveryError, TokenResponse as _};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use url::Url;

use crate::auth::{self, Result};

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

type GoogleIdToken = openidconnect::IdToken<
    EmptyAdditionalClaims,
    CoreGenderClaim,
    CoreJweContentEncryptionAlgorithm,
    CoreJwsSigningAlgorithm,
>;

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

    async fn discover_provider_metadata(
        &self,
        mm: &ModelManager,
    ) -> Result<GoogleProviderMetadata> {
        let issuer_url =
            IssuerUrl::new("https://accounts.google.com".to_string()).expect("valid issuer URL");
        let provider_metadata =
            GoogleProviderMetadata::discover_async(issuer_url, mm.http_client_wrapper()).await?;
        Ok(provider_metadata)
    }

    fn create_client(
        &self,
        provider_metadata: GoogleProviderMetadata,
        redirect_uri: Url,
    ) -> Result<GoogleCoreClient> {
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
        Ok(client)
    }
}

#[async_trait]
impl AuthProvider for GoogleAuthProvider {
    fn name(&self) -> &'static str {
        "google"
    }

    async fn authorize_url(&self, mm: &ModelManager, redirect_uri: Url) -> Result<AuthorizeUrl> {
        let provider_metadata = self.discover_provider_metadata(mm).await?;
        let client = self.create_client(provider_metadata, redirect_uri)?;

        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let (authorize_url, csrf_state, nonce) = client
            .authorize_url(
                AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
            .set_pkce_challenge(pkce_challenge)
            .add_scope(Scope::new("email".to_string()))
            .add_scope(Scope::new("profile".to_string()))
            .url();

        Ok(AuthorizeUrl {
            url: authorize_url,
            state: AuthFlowState {
                csrf_state: Some(csrf_state),
                nonce: Some(nonce),
                pkce_verifier: Some(pkce_verifier),
            },
        })
    }

    async fn exchange_code(
        &self,
        mm: &ModelManager,
        code: String,
        redirect_uri: Url,
        pkce_verifier: Option<PkceCodeVerifier>,
    ) -> Result<(TokenResponse, IdToken)> {
        let provider_metadata = self.discover_provider_metadata(mm).await?;
        let client = self.create_client(provider_metadata, redirect_uri.clone())?;

        let mut req = client
            .exchange_code(AuthorizationCode::new(code.to_string()))?
            .set_redirect_uri(Cow::Owned(RedirectUrl::from_url(redirect_uri)));

        if let Some(pkce_verifier) = pkce_verifier {
            req = req.set_pkce_verifier(pkce_verifier);
        }

        let token_response = req.request_async(mm.http_client_wrapper()).await?;

        let id_token = token_response
            .id_token()
            .cloned()
            .map(|id_token| IdToken(Box::new(id_token)))
            .expect("Google always returns ID tokens");

        let tokens = TokenResponse {
            access_token: token_response.access_token().clone(),
            refresh_token: token_response.refresh_token().cloned(),
            expires_at: token_response
                .expires_in()
                .map(|dur| OffsetDateTime::now_utc() + dur),
        };

        Ok((tokens, id_token))
    }

    async fn fetch_identity(
        &self,
        mm: &ModelManager,
        id_token: IdToken,
        nonce: Option<Nonce>,
        redirect_uri: Url,
    ) -> Result<UserIdentity> {
        let provider_metadata = self.discover_provider_metadata(mm).await?;
        let client = self.create_client(provider_metadata, redirect_uri)?;

        let id_token = id_token
            .0
            .downcast::<GoogleIdToken>()
            .map_err(|_| auth::Error::InvalidIdTokenType)?;

        let verifier = client.id_token_verifier();
        let claims = id_token.claims(
            &verifier,
            &nonce.expect("nonce is required for Google ID tokens"),
        )?;

        Ok(UserIdentity {
            id: claims.subject().to_string(),
            username: claims.preferred_username().map(|u| u.to_string()),
            email: claims.email().map(|e| e.to_string()),
            email_verified: claims.email_verified().unwrap_or(false),
            profile_picture: claims.picture().and_then(|urls| {
                urls.iter()
                    .find_map(|(_, url)| Url::parse(url.as_str()).ok())
            }),
        })
    }
}
