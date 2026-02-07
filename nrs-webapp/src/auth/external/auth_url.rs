use super::Result;
use oauth2::{
    CsrfToken, EndpointMaybeSet, EndpointSet, EndpointState, ErrorResponse, PkceCodeChallenge,
    PkceCodeVerifier, RevocableToken, Scope, TokenIntrospectionResponse, TokenResponse,
};
use openidconnect::{
    AdditionalClaims, AuthDisplay, AuthPrompt, AuthenticationFlow, GenderClaim, JsonWebKey,
    JweContentEncryptionAlgorithm, JwsSigningAlgorithm, Nonce, core::CoreResponseType,
};
use serde::{Deserialize, Serialize};
use url::Url;

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

pub trait AuthorizeUrlGenerator {
    fn create_authorize_url(&self) -> Result<AuthorizeUrl>;
}

pub trait BaseAuthorizeUrlGenerator {
    type Client;

    fn as_client(&self) -> &Self::Client;

    fn scopes(&self) -> &'static [&'static str] {
        &[]
    }

    fn create_csrf_token() -> CsrfToken {
        CsrfToken::new_random()
    }

    fn create_nonce() -> Nonce {
        Nonce::new_random()
    }

    fn create_pkce_challenge() -> Option<(PkceCodeChallenge, PkceCodeVerifier)> {
        Some(PkceCodeChallenge::new_random_sha256())
    }
}

pub struct OAuthAuthorizeUrlGenerator<'a, G>(&'a G);
pub struct OidcAuthorizeUrlGenerator<'a, G>(&'a G);

pub(super) trait OAuthAuthorizeUrlGeneratorTrait {
    fn oauth(&self) -> OAuthAuthorizeUrlGenerator<'_, Self>
    where
        Self: Sized,
    {
        OAuthAuthorizeUrlGenerator(self)
    }

    fn create_authorize_url_oauth<'a>(&'a self) -> Result<AuthorizeUrl>
    where
        OAuthAuthorizeUrlGenerator<'a, Self>: AuthorizeUrlGenerator,
        Self: Sized,
    {
        self.oauth().create_authorize_url()
    }
}
impl<G> OAuthAuthorizeUrlGeneratorTrait for G {}

pub(super) trait OidcAuthorizeUrlGeneratorTrait {
    fn oidc(&self) -> OidcAuthorizeUrlGenerator<'_, Self>
    where
        Self: Sized,
    {
        OidcAuthorizeUrlGenerator(self)
    }

    fn create_authorize_url_oidc<'a>(&'a self) -> Result<AuthorizeUrl>
    where
        OidcAuthorizeUrlGenerator<'a, Self>: AuthorizeUrlGenerator,
        Self: Sized,
    {
        self.oidc().create_authorize_url()
    }
}
impl<G> OidcAuthorizeUrlGeneratorTrait for G {}

impl<
    'a,
    G,
    TE,
    TR,
    TIR,
    RT,
    TRE,
    HasDeviceAuthUrl,
    HasIntrospectionUrl,
    HasRevocationUrl,
    HasTokenUrl,
> AuthorizeUrlGenerator for OAuthAuthorizeUrlGenerator<'a, G>
where
    G: BaseAuthorizeUrlGenerator<
        Client = oauth2::Client<
            TE,
            TR,
            TIR,
            RT,
            TRE,
            EndpointMaybeSet,
            HasDeviceAuthUrl,
            HasIntrospectionUrl,
            HasRevocationUrl,
            HasTokenUrl,
        >,
    >,
    TE: ErrorResponse + 'static,
    TR: TokenResponse,
    TIR: TokenIntrospectionResponse,
    RT: RevocableToken,
    TRE: ErrorResponse + 'static,
    HasDeviceAuthUrl: EndpointState,
    HasIntrospectionUrl: EndpointState,
    HasRevocationUrl: EndpointState,
    HasTokenUrl: EndpointState,
{
    fn create_authorize_url(&self) -> Result<AuthorizeUrl> {
        let inner = self.0;
        let client = inner.as_client();
        let mut req = client.authorize_url(|| G::create_csrf_token())?;

        let pkce_verifier =
            if let Some((pkce_challenge, pkce_verifier)) = G::create_pkce_challenge() {
                req = req.set_pkce_challenge(pkce_challenge);
                Some(pkce_verifier)
            } else {
                None
            };

        for scope in inner.scopes() {
            req = req.add_scope(Scope::new(scope.to_string()));
        }

        let (authorize_url, csrf_state) = req.url();

        Ok(AuthorizeUrl {
            url: authorize_url,
            state: AuthFlowState {
                csrf_state: Some(csrf_state),
                nonce: None,
                pkce_verifier,
            },
        })
    }
}

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
    HasDeviceAuthUrl,
    HasIntrospectionUrl,
    HasRevocationUrl,
    HasTokenUrl,
    HasUserInfoUrl,
> AuthorizeUrlGenerator for OidcAuthorizeUrlGenerator<'a, G>
where
    G: BaseAuthorizeUrlGenerator<
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
            EndpointSet,
            HasDeviceAuthUrl,
            HasIntrospectionUrl,
            HasRevocationUrl,
            HasTokenUrl,
            HasUserInfoUrl,
        >,
    >,
    AC: AdditionalClaims,
    AD: AuthDisplay,
    GC: GenderClaim,
    JE: JweContentEncryptionAlgorithm<
        KeyType = <K::SigningAlgorithm as JwsSigningAlgorithm>::KeyType,
    >,
    K: JsonWebKey,
    P: AuthPrompt,
    TE: ErrorResponse + 'static,
    TR: openidconnect::TokenResponse<AC, GC, JE, K::SigningAlgorithm>,
    TIR: TokenIntrospectionResponse,
    RT: RevocableToken,
    TRE: ErrorResponse + 'static,
    HasDeviceAuthUrl: EndpointState,
    HasIntrospectionUrl: EndpointState,
    HasRevocationUrl: EndpointState,
    HasTokenUrl: EndpointState,
    HasUserInfoUrl: EndpointState,
{
    fn create_authorize_url(&self) -> Result<AuthorizeUrl> {
        let inner = self.0;
        let client = inner.as_client();
        let mut req = client.authorize_url(
            // TODO: support other response types?
            AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
            || G::create_csrf_token(),
            || G::create_nonce(),
        );

        let pkce_verifier =
            if let Some((pkce_challenge, pkce_verifier)) = G::create_pkce_challenge() {
                req = req.set_pkce_challenge(pkce_challenge);
                Some(pkce_verifier)
            } else {
                None
            };

        for scope in inner.scopes() {
            req = req.add_scope(Scope::new(scope.to_string()));
        }

        let (authorize_url, csrf_state, nonce) = req.url();

        Ok(AuthorizeUrl {
            url: authorize_url,
            state: AuthFlowState {
                csrf_state: Some(csrf_state),
                nonce: Some(nonce),
                pkce_verifier,
            },
        })
    }
}
