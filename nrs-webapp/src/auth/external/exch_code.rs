use std::any::Any;

use super::{Error, Result};
use crate::model::{HttpClientWrapper, OAuth2HttpClientError};
use async_trait::async_trait;
use oauth2::{
    AccessToken, AuthorizationCode, EndpointMaybeSet, EndpointState, ErrorResponse, RefreshToken,
    RequestTokenError, RevocableToken, TokenIntrospectionResponse,
};
use openidconnect::{
    AdditionalClaims, AuthDisplay, AuthPrompt, GenderClaim, JsonWebKey,
    JweContentEncryptionAlgorithm, JwsSigningAlgorithm,
};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

pub struct IdToken(pub(super) Box<dyn Any + Send + Sync>);

impl Default for IdToken {
    fn default() -> Self {
        IdToken(Box::new(()))
    }
}

#[derive(Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: AccessToken,
    pub refresh_token: Option<RefreshToken>,
    pub expires_at: Option<OffsetDateTime>,
}

#[async_trait]
pub trait CodeExchanger {
    type TR;

    async fn exchange_code(
        &self,
        http_client: &HttpClientWrapper,
        code: String,
        pkce_verifier: Option<oauth2::PkceCodeVerifier>,
    ) -> Result<(TokenResponse, IdToken)>;

    fn get_id_token(&self, _token_resp: Self::TR) -> Result<IdToken> {
        Ok(IdToken::default())
    }
}

pub trait BaseCodeExchanger {
    type Client;

    fn as_client(&self) -> &Self::Client;
}

pub struct OAuthBaseCodeExchanger<'a, G>(&'a G);
pub struct OidcBaseCodeExchanger<'a, G>(&'a G);

pub(super) trait OAuthCodeExchangerTrait {
    fn oauth(&self) -> OAuthBaseCodeExchanger<'_, Self>
    where
        Self: Sized,
    {
        OAuthBaseCodeExchanger(self)
    }

    async fn exchange_code_oauth<'a>(
        &'a self,
        http_client: &HttpClientWrapper,
        code: String,
        pkce_verifier: Option<oauth2::PkceCodeVerifier>,
    ) -> Result<(TokenResponse, IdToken)>
    where
        OAuthBaseCodeExchanger<'a, Self>: CodeExchanger,
        Self: Sized,
    {
        self.oauth()
            .exchange_code(http_client, code, pkce_verifier)
            .await
    }
}
pub(super) trait OidcCodeExchangerTrait {
    fn oidc(&self) -> OidcBaseCodeExchanger<'_, Self>
    where
        Self: Sized,
    {
        OidcBaseCodeExchanger(self)
    }

    async fn exchange_code_oidc<'a>(
        &'a self,
        http_client: &HttpClientWrapper,
        code: String,
        pkce_verifier: Option<oauth2::PkceCodeVerifier>,
    ) -> Result<(TokenResponse, IdToken)>
    where
        OidcBaseCodeExchanger<'a, Self>: CodeExchanger,
        Self: Sized,
    {
        self.oidc()
            .exchange_code(http_client, code, pkce_verifier)
            .await
    }
}
impl<G> OAuthCodeExchangerTrait for G {}
impl<G> OidcCodeExchangerTrait for G {}

#[async_trait]
impl<
    'a,
    G,
    TE,
    TR,
    TIR,
    RT,
    TRE,
    HasAuthUrl,
    HasDeviceAuthUrl,
    HasIntrospectionUrl,
    HasRevocationUrl,
> CodeExchanger for OAuthBaseCodeExchanger<'a, G>
where
    G: BaseCodeExchanger<
            Client = oauth2::Client<
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
            >,
        > + Sync,
    TE: ErrorResponse + Send + 'static,
    TR: oauth2::TokenResponse + Send,
    TIR: TokenIntrospectionResponse,
    RT: RevocableToken,
    TRE: ErrorResponse + 'static,
    HasAuthUrl: EndpointState,
    HasDeviceAuthUrl: EndpointState,
    HasIntrospectionUrl: EndpointState,
    HasRevocationUrl: EndpointState,
    Error: From<RequestTokenError<OAuth2HttpClientError, TE>>,
{
    type TR = TR;

    async fn exchange_code(
        &self,
        http_client: &HttpClientWrapper,
        code: String,
        pkce_verifier: Option<oauth2::PkceCodeVerifier>,
    ) -> Result<(TokenResponse, IdToken)> {
        let inner = self.0;
        let client = inner.as_client();
        let mut req = client.exchange_code(AuthorizationCode::new(code))?;

        if let Some(pkce_verifier) = pkce_verifier {
            req = req.set_pkce_verifier(pkce_verifier);
        }

        let token_response = req.request_async(http_client).await?;

        let tokens = TokenResponse {
            access_token: token_response.access_token().clone(),
            refresh_token: token_response.refresh_token().cloned(),
            expires_at: token_response
                .expires_in()
                .map(|dur| OffsetDateTime::now_utc() + dur),
        };

        Ok((tokens, self.get_id_token(token_response)?))
    }
}

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
> CodeExchanger for OidcBaseCodeExchanger<'a, G>
where
    G: BaseCodeExchanger<
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
    type TR = TR;

    async fn exchange_code(
        &self,
        http_client: &HttpClientWrapper,
        code: String,
        pkce_verifier: Option<oauth2::PkceCodeVerifier>,
    ) -> Result<(TokenResponse, IdToken)> {
        let inner = self.0;
        let client = inner.as_client();
        let mut req = client.exchange_code(AuthorizationCode::new(code))?;

        if let Some(pkce_verifier) = pkce_verifier {
            req = req.set_pkce_verifier(pkce_verifier);
        }

        let token_response = req.request_async(http_client).await?;

        let tokens = TokenResponse {
            access_token: token_response.access_token().clone(),
            refresh_token: token_response.refresh_token().cloned(),
            expires_at: token_response
                .expires_in()
                .map(|dur| OffsetDateTime::now_utc() + dur),
        };

        Ok((tokens, self.get_id_token(token_response)?))
    }
}
