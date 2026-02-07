use always_send::FutureExt;
use anyhow::Context;
use axum::{
    Router,
    extract::{Path, Query, State},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
};
use axum_client_ip::ClientIp;
use axum_extra::{
    TypedHeader,
    extract::{PrivateCookieJar, SignedCookieJar},
    headers::UserAgent,
};
use axum_htmx::{HxRedirect, HxRequest};
use nrs_webapp_frontend::{
    maybe_document,
    views::{
        document::DocumentProps,
        pages::auth::register::{RegisterScreen, register},
    },
};
use oauth2::CsrfToken;
use serde::Deserialize;

use crate::{
    Error, Result,
    auth::{
        TempTokensCookie, add_auth_cookie, add_auth_flow_state_cookie, add_temp_tokens_cookie,
        external::{
            UserIdentity,
            auth_url::{AuthFlowState, AuthorizeUrl},
        },
        get_auth_flow_state_cookie, get_temp_tokens_cookie, remove_auth_flow_state_cookie,
        remove_temp_tokens_cookie,
    },
    config::AppConfig,
    crypt::{
        password_hash::PasswordHasher, session_token::SessionToken, symmetric::SymmetricCipher,
    },
    extract::with_rejection::WRVForm,
    model::{
        entity::DbBmc,
        oauth_links::{OAuthLinkBmc, OAuthLinkForCreate, OAuthLinkForUpdate},
        user::{UserBmc, UserForCreate},
    },
    routes::auth::{confirm_mail::redirect_to_confirm_mail_page, register::RegisterPayload},
};
use crate::{auth, model::ModelManager};

pub fn router() -> Router<ModelManager> {
    Router::new()
        .route("/authorize/{provider}", get(authorize_handler))
        .route("/callback/{provider}", get(callback_handler))
        .route("/register", post(register_handler))
}

fn build_redirect_uri(provider_name: &str) -> Result<url::Url> {
    AppConfig::get()
        .SERVICE_BASE_URL
        .clone()
        .join("/auth/oauth/callback/")
        .and_then(|u| u.join(provider_name))
        .context("invalid redirect url")
        .map_err(Error::Unexpected)
}

async fn authorize_handler(
    Path(provider): Path<String>,
    secret_jar: PrivateCookieJar,
    State(mm): State<ModelManager>,
) -> Result<impl IntoResponse> {
    tracing::debug!(
        "{:<12} -- GET auth::oauth::authorize_handler {}",
        "ROUTE",
        provider
    );

    let provider = mm
        .auth_providers()
        .get(&provider)
        .ok_or_else(|| Error::Auth(auth::Error::ProviderNotFound(provider)))?;

    let redirect_uri = build_redirect_uri(provider.name())?;

    tracing::debug!(
        "{:<12} -- Redirecting to OAuth2 provider {} authorize URL (redirect_uri={})",
        "ROUTE",
        provider.name(),
        redirect_uri
    );

    let AuthorizeUrl { url, state } = provider.authorize_url(&mm, redirect_uri).await?;

    Ok((
        add_auth_flow_state_cookie(secret_jar, &state)?,
        Redirect::to(url.as_ref()),
    ))
}

#[derive(Deserialize)]
struct CallbackQueryParams {
    code: String,
    state: String,
}

async fn callback_handler(
    jar: SignedCookieJar,
    secret_jar: PrivateCookieJar,
    Query(CallbackQueryParams { code, state }): Query<CallbackQueryParams>,
    Path(provider_name): Path<String>,
    State(mut mm): State<ModelManager>,
) -> Result<Response> {
    tracing::debug!(
        "{:<12} -- GET auth::oauth::callback_handler {}",
        "ROUTE",
        provider_name
    );

    let AuthFlowState {
        csrf_state,
        nonce,
        pkce_verifier,
    } = get_auth_flow_state_cookie(&secret_jar)
        .ok_or_else(|| auth::Error::AuthFlowStateCookieNotFound)?;

    if csrf_state
        .map(|s| s != CsrfToken::new(state))
        .unwrap_or(false)
    {
        return Err(Error::Auth(auth::Error::CsrfStateMismatch));
    }

    let provider = mm
        .auth_providers()
        .get(&provider_name)
        .ok_or_else(|| Error::Auth(auth::Error::ProviderNotFound(provider_name.clone())))?;

    let redirect_uri = build_redirect_uri(provider.name())?;

    let (tokens, id_token) = provider
        .exchange_code(&mm, code, redirect_uri.clone(), pkce_verifier)
        .await?;

    let UserIdentity {
        id,
        username,
        email,
        email_verified,
        ..
    } = provider
        .fetch_identity(&mm, id_token, nonce, &tokens.access_token, redirect_uri)
        .await?;

    let cipher = SymmetricCipher::get_from_config();
    let encrypted_access_token = cipher.encrypt(tokens.access_token.secret().as_bytes())?;
    let encrypted_refresh_token = tokens
        .refresh_token
        .as_ref()
        .map(|refresh_token| cipher.encrypt(refresh_token.secret().as_bytes()))
        .transpose()?;

    let user_id = OAuthLinkBmc::update_link(
        &mut mm,
        &provider_name,
        &id,
        OAuthLinkForUpdate {
            access_token: encrypted_access_token,
            refresh_token: encrypted_refresh_token,
            access_token_expires_at: tokens.expires_at,
        },
    )
    .await?;

    if let Some(user_id) = user_id {
        Ok((
            remove_auth_flow_state_cookie(secret_jar),
            add_auth_cookie(jar, SessionToken::new(user_id)),
            Redirect::to("/"),
        )
            .into_response())
    } else {
        Ok((
            add_temp_tokens_cookie(
                remove_auth_flow_state_cookie(secret_jar),
                TempTokensCookie {
                    tokens,
                    email: email.clone(),
                    email_verified,
                    subject: id,
                    provider_name,
                },
            ),
            maybe_document(
                HxRequest(false),
                DocumentProps::default(),
                register(RegisterScreen::OAuth { username, email }),
            ),
        )
            .into_response())
    }
}

async fn register_handler(
    jar: SignedCookieJar,
    secret_jar: PrivateCookieJar,
    State(mm): State<ModelManager>,
    ClientIp(ip_addr): ClientIp,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    WRVForm(RegisterPayload {
        username,
        email,
        password,
    }): WRVForm<RegisterPayload>,
) -> Result<Response> {
    tracing::debug!("{:<12} -- POST auth::oauth::register_handler", "ROUTE");

    let TempTokensCookie {
        tokens,
        email: email_cookie,
        email_verified,
        subject,
        provider_name,
    } = get_temp_tokens_cookie(&secret_jar).ok_or(auth::Error::TempTokenCookieNotFound)?;

    // make sure email == email_cookie (if email_cookie exists)
    if let Some(email_cookie) = email_cookie
        && email_cookie != email
    {
        return Err(Error::Auth(auth::Error::EmailMismatch));
    }

    let password_hash = PasswordHasher::get_from_config().encrypt_password(&password)?;

    let mut tx = mm.tx().await?;

    let user_id = UserBmc::create_user(
        &mut tx,
        UserForCreate {
            username: username.clone(),
            email,
            password_hash,
        },
    )
    .always_send()
    .await?;

    if email_verified {
        UserBmc::mark_email_verified(&mut tx, user_id)
            .always_send()
            .await?;
    }

    let cipher = SymmetricCipher::get_from_config();
    let encrypted_access_token = cipher.encrypt(tokens.access_token.secret().as_bytes())?;
    let encrypted_refresh_token = tokens
        .refresh_token
        .as_ref()
        .map(|refresh_token| cipher.encrypt(refresh_token.secret().as_bytes()))
        .transpose()?;

    OAuthLinkBmc::create(
        &mut tx,
        OAuthLinkForCreate {
            user_id,
            provider: provider_name,
            provider_user_id: Some(subject),
            access_token: encrypted_access_token,
            refresh_token: encrypted_refresh_token,
            access_token_expires_at: tokens.expires_at,
        },
    )
    .always_send()
    .await?;

    tx.commit().always_send().await?;

    if email_verified {
        Ok((
            HxRedirect("/".into()),
            add_auth_cookie(jar, SessionToken::new(user_id)),
            remove_temp_tokens_cookie(secret_jar),
        )
            .into_response())
    } else {
        Ok((
            remove_temp_tokens_cookie(secret_jar),
            redirect_to_confirm_mail_page(mm, username, ip_addr, user_agent),
        )
            .into_response())
    }
}
