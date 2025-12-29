pub mod error;
pub mod session;

use std::time::Duration;

use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, Expiration, SameSite},
};
pub use error::{Error, Result};

use crate::config::AppConfig;

const AUTH_COOKIE_NAME: &str = "nrs_auth_token";

pub fn add_auth_cookie(jar: CookieJar, token: String) -> CookieJar {
    jar.add(
        Cookie::build((AUTH_COOKIE_NAME, token))
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

pub fn remove_auth_cookie(jar: CookieJar) -> CookieJar {
    jar.remove(Cookie::build(AUTH_COOKIE_NAME).path("/"))
}

pub fn get_auth_cookie(jar: &CookieJar) -> Option<String> {
    jar.get(AUTH_COOKIE_NAME).map(|c| c.value().to_string())
}
