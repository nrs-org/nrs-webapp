pub mod error;
pub mod session;

use axum_extra::extract::{
    SignedCookieJar,
    cookie::{Cookie, SameSite},
};
pub use error::{Error, Result};

use crate::{config::AppConfig, crypt::session_token::SessionToken};

const AUTH_COOKIE_NAME: &str = "nrs_auth_token";

/// Adds an authentication cookie with the provided token to the given `CookieJar`.
///
/// The cookie is named "nrs_auth_token" and is configured as HTTP-only, uses `SameSite::Lax`,
/// has path "/", and has a max-age derived from `AppConfig::get().SERVICE_JWT_EXPIRY_DURATION`.
/// The cookie's `secure` flag is enabled in non-debug builds.
///
/// # Examples
///
/// ```
/// use axum_extra::extract::CookieJar;
///
/// let jar = CookieJar::new();
/// let jar = nrs_webapp::auth::add_auth_cookie(jar, "token123".to_string());
/// let cookie = jar.get("nrs_auth_token").expect("cookie should be present");
/// assert_eq!(cookie.value(), "token123");
/// ```
pub fn add_auth_cookie(jar: SignedCookieJar, token: SessionToken) -> SignedCookieJar {
    jar.add(
        Cookie::build((AUTH_COOKIE_NAME, token.to_string()))
            .http_only(true)
            .secure(!cfg!(debug_assertions))
            .same_site(SameSite::Lax)
            .path("/")
            .max_age(
                time::Duration::try_from(AppConfig::get().SERVICE_JWT_EXPIRY_DURATION)
                    .expect("negative duration"),
            ),
    )
}

/// Removes the authentication cookie from the provided cookie jar.
///
/// # Returns
///
/// The updated `CookieJar` with the authentication cookie removed.
///
/// # Examples
///
/// ```
/// use axum_extra::extract::CookieJar;
///
/// let jar = CookieJar::new();
/// let jar = remove_auth_cookie(jar);
/// ```
pub fn remove_auth_cookie(jar: SignedCookieJar) -> SignedCookieJar {
    jar.remove(Cookie::build(AUTH_COOKIE_NAME).path("/"))
}

/// Retrieve the authentication cookie value from a `CookieJar`.
///
/// # Returns
///
/// `Some(String)` with the cookie value if present, `None` otherwise.
///
/// # Examples
///
/// ```no_run
/// use axum_extra::extract::CookieJar;
/// use nrs_webapp::auth::get_auth_cookie;
///
/// let jar = CookieJar::new();
/// // assume a cookie named "nrs_auth_token" was previously added to `jar`
/// let value = get_auth_cookie(&jar);
/// assert!(value.is_none() || value.is_some());
/// ```
pub fn get_auth_cookie(jar: &SignedCookieJar) -> Option<String> {
    jar.get(AUTH_COOKIE_NAME).map(|c| c.value().to_string())
}
