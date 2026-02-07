pub mod error;
pub mod external;
pub mod session;

use axum_extra::extract::{
    PrivateCookieJar, SignedCookieJar,
    cookie::{Cookie, SameSite},
};
pub use error::{Error, Result};
use serde::{Deserialize, Serialize};

use crate::{
    auth::external::{auth_url::AuthFlowState, exch_code::TokenResponse},
    config::AppConfig,
    crypt::session_token::SessionToken,
};

const AUTH_COOKIE_NAME: &str = "nrs_auth_token";
const AUTH_FLOW_STATE_COOKIE_NAME: &str = "nrs_auth_flow_state";
const AUTH_TEMP_TOKENS_COOKIE_NAME: &str = "nrs_temp_tokens";

/// Adds an authentication cookie with the provided token to the given `SignedCookieJar`.
///
/// The cookie is named "nrs_auth_token" and is configured as HTTP-only, uses `SameSite::Lax`,
/// has path "/", and has a max-age derived from `AppConfig::get().SERVICE_SESSION_EXPIRY_DURATION`.
/// The cookie's `secure` flag is enabled in non-debug builds.
///
/// # Examples
///
/// ```
/// use axum_extra::extract::SignedCookieJar;
/// use cookie::Key;
/// use nrs_webapp::auth::{add_auth_cookie, SessionToken};
/// use uuid::Uuid;
///
/// let jar = SignedCookieJar::new(Key::generate());
/// let user_id = Uuid::new_v4();
/// let token = SessionToken::new(user_id);
/// let jar = nrs_webapp::auth::add_auth_cookie(jar, token);
/// let _cookie = jar.get("nrs_auth_token").expect("cookie should be present");
/// ```
pub fn add_auth_cookie(jar: SignedCookieJar, token: SessionToken) -> SignedCookieJar {
    jar.add(
        Cookie::build((AUTH_COOKIE_NAME, token.to_string()))
            .http_only(true)
            .secure(!cfg!(debug_assertions))
            .same_site(SameSite::Lax)
            .path("/")
            .max_age(
                time::Duration::try_from(AppConfig::get().SERVICE_SESSION_EXPIRY_DURATION)
                    .expect("negative duration"),
            ),
    )
}

/// Removes the authentication cookie from the provided cookie jar.
///
/// # Returns
///
/// The updated `SignedCookieJar` with the authentication cookie removed.
///
/// # Examples
///
/// ```
/// use axum_extra::extract::SignedCookieJar;
/// use cookie::Key;
///
/// let jar = SignedCookieJar::new(Key::generate());
/// let jar = remove_auth_cookie(jar);
/// ```
pub fn remove_auth_cookie(jar: SignedCookieJar) -> SignedCookieJar {
    jar.remove(Cookie::build(AUTH_COOKIE_NAME).path("/"))
}

/// Retrieve the authentication cookie value from a `SignedCookieJar`.
///
/// # Returns
///
/// `Some(String)` with the cookie value if present, `None` otherwise.
///
/// # Examples
///
/// ```no_run
/// use axum_extra::extract::SignedCookieJar;
/// use cookie::Key;
/// use nrs_webapp::auth::get_auth_cookie;
///
/// let jar = SignedCookieJar::new(Key::generate());
/// // assume a cookie named "nrs_auth_token" was previously added to `jar`
/// let value = get_auth_cookie(&jar);
/// assert!(value.is_none() || value.is_some());
/// ```
pub fn get_auth_cookie(jar: &SignedCookieJar) -> Option<String> {
    jar.get(AUTH_COOKIE_NAME).map(|c| c.value().to_string())
}

pub fn add_auth_flow_state_cookie(
    jar: SignedCookieJar,
    auth_flow_state: &AuthFlowState,
) -> Result<SignedCookieJar> {
    let state_json = serde_json::to_string(auth_flow_state)?;
    Ok(jar.add(
        Cookie::build((AUTH_FLOW_STATE_COOKIE_NAME, state_json))
            .http_only(true)
            .secure(!cfg!(debug_assertions))
            .same_site(SameSite::Lax)
            .path("/auth/oauth")
            .max_age(
                time::Duration::try_from(AppConfig::get().SERVICE_OAUTH_EXPIRY_DURATION)
                    .expect("negative duration"),
            ),
    ))
}

pub fn remove_auth_flow_state_cookie(jar: SignedCookieJar) -> SignedCookieJar {
    jar.remove(Cookie::build(AUTH_FLOW_STATE_COOKIE_NAME).path("/auth/oauth"))
}

pub fn get_auth_flow_state_cookie(jar: &SignedCookieJar) -> Option<AuthFlowState> {
    if let Some(cookie) = jar.get(AUTH_FLOW_STATE_COOKIE_NAME) {
        match serde_json::from_str(cookie.value()) {
            Ok(state) => return Some(state),
            Err(err) => {
                tracing::warn!(
                    "Failed to deserialize auth flow state from cookie value: {}",
                    err
                );
            }
        }
    }

    None
}

#[derive(Serialize, Deserialize)]
pub struct TempTokensCookie {
    pub tokens: TokenResponse,
    pub email: Option<String>,
    pub email_verified: bool,
    pub subject: String,
    pub provider_name: String,
}

pub fn add_temp_tokens_cookie(jar: PrivateCookieJar, tokens: TempTokensCookie) -> PrivateCookieJar {
    let tokens_json = serde_json::to_string(&tokens).expect("should not fail on serialize");
    jar.add(
        Cookie::build((AUTH_TEMP_TOKENS_COOKIE_NAME, tokens_json))
            .http_only(true)
            .secure(!cfg!(debug_assertions))
            .same_site(SameSite::Lax)
            .path("/auth/oauth")
            .max_age(
                time::Duration::try_from(AppConfig::get().SERVICE_OAUTH_EXPIRY_DURATION)
                    .expect("negative duration"),
            ),
    )
}

pub fn remove_temp_tokens_cookie(jar: PrivateCookieJar) -> PrivateCookieJar {
    jar.remove(Cookie::build(AUTH_TEMP_TOKENS_COOKIE_NAME).path("/auth/oauth"))
}

pub fn get_temp_tokens_cookie(jar: &PrivateCookieJar) -> Option<TempTokensCookie> {
    if let Some(cookie) = jar.get(AUTH_TEMP_TOKENS_COOKIE_NAME) {
        match serde_json::from_str(cookie.value()) {
            Ok(tokens) => return Some(tokens),
            Err(err) => {
                tracing::warn!(
                    "Failed to deserialize temp tokens from cookie value: {}",
                    err
                );
            }
        }
    }

    None
}
