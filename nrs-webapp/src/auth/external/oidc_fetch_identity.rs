use async_trait::async_trait;
use oauth2::{
    AccessToken, EndpointMaybeSet, EndpointState, ErrorResponse, RequestTokenError, RevocableToken,
    TokenIntrospectionResponse,
};
use openidconnect::{
    AdditionalClaims, AuthDisplay, AuthPrompt, GenderClaim, JsonWebKey,
    JweContentEncryptionAlgorithm, JwsSigningAlgorithm, Nonce,
};
use url::Url;

use super::{Error, Result};
use crate::model::OAuth2HttpClientError;
use crate::{
    auth::external::{UserIdentity, exch_code::IdToken},
    model::HttpClientWrapper,
};

#[async_trait]
pub trait IdentityFetcher {
    async fn fetch_identity(
        &self,
        http_client: &HttpClientWrapper,
        id_token: &IdToken,
        access_token: &AccessToken,
        nonce: Option<Nonce>,
    ) -> Result<UserIdentity>;
}

pub trait BaseIdentityFetcher {
    type Client;

    fn as_client(&self) -> &Self::Client;
}

pub struct OidcBaseIdentityFetcher<'a, G>(&'a G);

pub(super) trait OidcIdentityFetcherTrait {
    fn oidc(&self) -> OidcBaseIdentityFetcher<'_, Self>
    where
        Self: Sized,
    {
        OidcBaseIdentityFetcher(self)
    }

    async fn fetch_identity_oidc<'a>(
        &'a self,
        http_client: &HttpClientWrapper,
        id_token: &IdToken,
        access_token: &AccessToken,
        nonce: Option<Nonce>,
    ) -> Result<UserIdentity>
    where
        OidcBaseIdentityFetcher<'a, Self>: IdentityFetcher,
        Self: Sized,
    {
        self.oidc()
            .fetch_identity(http_client, id_token, access_token, nonce)
            .await
    }
}

impl<G> OidcIdentityFetcherTrait for G {}

#[async_trait]
impl<
    'a,
    G,
    AC,
    AD,
    GC,
    JE,
    K,
    P,
    TE,
    TR,
    TIR,
    RT,
    TRE,
    HasAuthUrl,
    HasDeviceAuthUrl,
    HasIntrospectionUrl,
    HasRevocationUrl,
    HasUserInfoUrl,
> IdentityFetcher for OidcBaseIdentityFetcher<'a, G>
where
    G: BaseIdentityFetcher<
            Client = openidconnect::Client<
                AC,
                AD,
                GC,
                JE,
                K,
                P,
                TE,
                TR,
                TIR,
                RT,
                TRE,
                HasAuthUrl,
                HasDeviceAuthUrl,
                HasIntrospectionUrl,
                HasRevocationUrl,
                EndpointMaybeSet,
                HasUserInfoUrl,
            >,
        > + Sync,
    AC: AdditionalClaims,
    AD: AuthDisplay,
    GC: GenderClaim,
    JE: JweContentEncryptionAlgorithm<
        KeyType = <K::SigningAlgorithm as JwsSigningAlgorithm>::KeyType,
    >,
    K: JsonWebKey,
    P: AuthPrompt,
    TE: ErrorResponse + Send + 'static,
    TR: openidconnect::TokenResponse<AC, GC, JE, K::SigningAlgorithm> + Send,
    TIR: TokenIntrospectionResponse,
    RT: RevocableToken,
    TRE: ErrorResponse + 'static,
    HasAuthUrl: EndpointState,
    HasDeviceAuthUrl: EndpointState,
    HasIntrospectionUrl: EndpointState,
    HasRevocationUrl: EndpointState,
    HasUserInfoUrl: EndpointState,
    Error: From<RequestTokenError<OAuth2HttpClientError, TE>>,
{
    async fn fetch_identity(
        &self,
        _http_client: &HttpClientWrapper,
        id_token: &IdToken,
        _access_token: &AccessToken,
        nonce: Option<Nonce>,
    ) -> Result<UserIdentity> {
        let inner = self.0;
        let client = inner.as_client();
        let verifier = client.id_token_verifier();

        let id_token = id_token
            .0
            .downcast_ref::<openidconnect::IdToken<AC, GC, JE, K::SigningAlgorithm>>()
            .ok_or_else(|| Error::InvalidIdTokenType)?;

        let claims = id_token.claims(&verifier, &nonce.ok_or(Error::NonceMissing)?)?;
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
