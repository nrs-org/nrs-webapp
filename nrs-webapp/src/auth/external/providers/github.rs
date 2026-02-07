use crate::auth::external::{
    AuthFlowState, AuthProvider, AuthorizeUrl, IdToken, TokenResponse, UserIdentity,
};
use crate::model::ModelManager;
use async_trait::async_trait;
use oauth2::basic::{BasicClient, BasicErrorResponseType, BasicTokenType};
use oauth2::{
    AccessToken, AuthUrl, AuthorizationCode, Client, EmptyExtraTokenFields, EndpointNotSet,
    EndpointSet, PkceCodeChallenge, PkceCodeVerifier, RevocationErrorResponseType,
    StandardErrorResponse, StandardRevocableToken, StandardTokenIntrospectionResponse,
    StandardTokenResponse, TokenResponse as _, TokenUrl,
};
use openidconnect::{ClientId, ClientSecret, CsrfToken, Nonce, RedirectUrl, Scope};
use reqwest_middleware::ClientWithMiddleware;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use time::OffsetDateTime;
use url::Url;

use crate::auth::Result;

type GithubCoreClient = Client<
    StandardErrorResponse<BasicErrorResponseType>,
    StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
    StandardTokenIntrospectionResponse<EmptyExtraTokenFields, BasicTokenType>,
    StandardRevocableToken,
    StandardErrorResponse<RevocationErrorResponseType>,
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointSet,
>;

pub struct GithubAuthProvider {
    client_id: String,
    client_secret: String,
}

impl GithubAuthProvider {
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            client_id,
            client_secret,
        }
    }

    pub fn from_config() -> Option<Self> {
        let config = crate::config::AppConfig::get()
            .GITHUB_OAUTH_CREDENTIALS
            .as_ref()?;
        Some(Self::new(
            config.client_id.clone(),
            config.client_secret.clone(),
        ))
    }

    fn create_client(&self, redirect_uri: Url) -> Result<GithubCoreClient> {
        let client = BasicClient::new(ClientId::new(self.client_id.clone()))
            .set_client_secret(ClientSecret::new(self.client_secret.clone()))
            .set_auth_uri(
                AuthUrl::new("https://github.com/login/oauth/authorize".into())
                    .expect("should be valid URL"),
            )
            .set_token_uri(
                TokenUrl::new("https://github.com/login/oauth/access_token".into())
                    .expect("should be valid URL"),
            )
            .set_redirect_uri(RedirectUrl::from_url(redirect_uri));
        Ok(client)
    }
}

#[async_trait]
impl AuthProvider for GithubAuthProvider {
    fn name(&self) -> &'static str {
        "github"
    }

    async fn authorize_url(&self, _mm: &ModelManager, redirect_uri: Url) -> Result<AuthorizeUrl> {
        let client = self.create_client(redirect_uri)?;

        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let (authorize_url, csrf_state) = client
            .authorize_url(CsrfToken::new_random)
            .set_pkce_challenge(pkce_challenge)
            .add_scope(Scope::new("user:email".to_string()))
            .add_scope(Scope::new("read:user".to_string()))
            .url();

        Ok(AuthorizeUrl {
            url: authorize_url,
            state: AuthFlowState {
                csrf_state: Some(csrf_state),
                nonce: None,
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
        let client = self.create_client(redirect_uri.clone())?;

        let mut req = client.exchange_code(AuthorizationCode::new(code.to_string()));

        if let Some(pkce_verifier) = pkce_verifier {
            req = req.set_pkce_verifier(pkce_verifier);
        }

        let token_response = req.request_async(mm.http_client_wrapper()).await?;

        let tokens = TokenResponse {
            access_token: token_response.access_token().clone(),
            refresh_token: token_response.refresh_token().cloned(),
            expires_at: token_response
                .expires_in()
                .map(|dur| OffsetDateTime::now_utc() + dur),
        };

        Ok((tokens, IdToken(Box::new(()))))
    }

    async fn fetch_identity(
        &self,
        mm: &ModelManager,
        _id_token: IdToken,
        _nonce: Option<Nonce>,
        access_token: &AccessToken,
        _redirect_uri: Url,
    ) -> Result<UserIdentity> {
        async fn http_get<E: DeserializeOwned>(
            client: &ClientWithMiddleware,
            endpoint: &str,
            access_token: &AccessToken,
        ) -> Result<E> {
            Ok(client
                .get(endpoint)
                .header("Accept", "application/vnd.github.v3+json")
                .header("User-Agent", "nrs-webapp")
                .bearer_auth(access_token.secret())
                .send()
                .await?
                .error_for_status()?
                .json::<E>()
                .await?)
        }

        #[derive(Debug, Deserialize)]
        struct User {
            id: u64,
            login: String,
            avatar_url: String,
        }

        #[derive(Debug, Deserialize)]
        struct UserEmail {
            email: String,
            primary: bool,
            verified: bool,
        }

        let user: User = http_get(
            mm.http_client(),
            "https://api.github.com/user",
            access_token,
        )
        .await?;

        tracing::debug!("GitHub user info: {:?}", user);

        let emails: Vec<UserEmail> = http_get(
            mm.http_client(),
            "https://api.github.com/user/emails",
            access_token,
        )
        .await?;

        tracing::debug!("GitHub user emails: {:?}", emails);

        // TODO: better email selection logic?
        // TODO: allow the user to select which email?
        let email = emails
            .into_iter()
            .enumerate()
            .min_by_key(|(idx, e)| {
                (
                    !e.verified, // prefer verified
                    !e.primary,  // then primary
                    *idx,        // then first in list
                )
            })
            .map(|(_, e)| e.email);

        Ok(UserIdentity {
            id: user.id.to_string(),
            username: Some(user.login),
            email,
            email_verified: true,
            profile_picture: Some(Url::parse(&user.avatar_url)?),
        })
    }
}
