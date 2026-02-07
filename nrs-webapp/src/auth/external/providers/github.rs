use crate::auth::external::auth_url::{
    AuthorizeUrl, BaseAuthorizeUrlGenerator, OAuthAuthorizeUrlGeneratorTrait,
};
use crate::auth::external::exch_code::{BaseCodeExchanger, OAuthCodeExchangerTrait};
use crate::auth::external::oidc_fetch_identity::IdentityFetcher;
use crate::auth::external::{AuthProvider, IdToken, TokenResponse, UserIdentity};
use crate::model::{HttpClientWrapper, ModelManager};
use async_trait::async_trait;
use oauth2::basic::{BasicClient, BasicErrorResponseType, BasicTokenType};
use oauth2::{
    AccessToken, AuthUrl, Client, EmptyExtraTokenFields, EndpointMaybeSet, EndpointNotSet,
    PkceCodeVerifier, RevocationErrorResponseType, StandardErrorResponse, StandardRevocableToken,
    StandardTokenIntrospectionResponse, StandardTokenResponse, TokenUrl,
};
use openidconnect::{ClientId, ClientSecret, Nonce, RedirectUrl};
use reqwest_middleware::ClientWithMiddleware;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use url::Url;

use crate::auth::Result;

type GithubCoreClient = Client<
    StandardErrorResponse<BasicErrorResponseType>,
    StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
    StandardTokenIntrospectionResponse<EmptyExtraTokenFields, BasicTokenType>,
    StandardRevocableToken,
    StandardErrorResponse<RevocationErrorResponseType>,
    EndpointMaybeSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointMaybeSet,
>;

struct GithubCoreClientWrapper(GithubCoreClient);

impl BaseAuthorizeUrlGenerator for GithubCoreClientWrapper {
    type Client = GithubCoreClient;

    fn as_client(&self) -> &Self::Client {
        &self.0
    }

    fn scopes(&self) -> &'static [&'static str] {
        &["user:email", "read:user"]
    }
}

impl BaseCodeExchanger for GithubCoreClientWrapper {
    type Client = GithubCoreClient;

    fn as_client(&self) -> &Self::Client {
        &self.0
    }
}

#[async_trait]
impl IdentityFetcher for GithubCoreClientWrapper {
    async fn fetch_identity(
        &self,
        http_client: &HttpClientWrapper,
        _id_token: &IdToken,
        access_token: &AccessToken,
        _nonce: Option<Nonce>,
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

        let user: User = http_get(http_client, "https://api.github.com/user", access_token).await?;

        let emails: Vec<UserEmail> = http_get(
            http_client,
            "https://api.github.com/user/emails",
            access_token,
        )
        .await?;

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
            .map(|(_, e)| (e.email, e.verified));

        Ok(UserIdentity {
            id: user.id.to_string(),
            username: Some(user.login),
            email_verified: email.as_ref().map(|(_, v)| *v).unwrap_or_default(),
            email: email.map(|(e, _)| e),
            profile_picture: Some(Url::parse(&user.avatar_url)?),
        })
    }
}

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

    fn create_client(&self, redirect_uri: Url) -> Result<GithubCoreClientWrapper> {
        let client = BasicClient::new(ClientId::new(self.client_id.clone()))
            .set_client_secret(ClientSecret::new(self.client_secret.clone()))
            .set_auth_uri_option(Some(
                AuthUrl::new("https://github.com/login/oauth/authorize".into())
                    .expect("should be valid URL"),
            ))
            .set_token_uri_option(Some(
                TokenUrl::new("https://github.com/login/oauth/access_token".into())
                    .expect("should be valid URL"),
            ))
            .set_redirect_uri(RedirectUrl::from_url(redirect_uri));
        Ok(GithubCoreClientWrapper(client))
    }
}

#[async_trait]
impl AuthProvider for GithubAuthProvider {
    fn name(&self) -> &'static str {
        "github"
    }

    async fn authorize_url(&self, _mm: &ModelManager, redirect_uri: Url) -> Result<AuthorizeUrl> {
        self.create_client(redirect_uri)?
            .create_authorize_url_oauth()
    }

    async fn exchange_code(
        &self,
        mm: &ModelManager,
        code: String,
        redirect_uri: Url,
        pkce_verifier: Option<PkceCodeVerifier>,
    ) -> Result<(TokenResponse, IdToken)> {
        self.create_client(redirect_uri)?
            .exchange_code_oauth(mm.http_client_wrapper(), code, pkce_verifier)
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
        self.create_client(redirect_uri)?
            .fetch_identity(mm.http_client_wrapper(), &id_token, access_token, nonce)
            .await
    }
}
