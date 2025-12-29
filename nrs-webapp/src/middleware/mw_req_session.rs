use axum::{extract::Request, middleware::Next, response::Response};
use axum_extra::extract::CookieJar;
use jsonwebtoken::TokenData;

use crate::{
    auth::{get_auth_cookie, session::Session},
    crypt::jwt::JwtContext,
};

pub async fn mw_req_session(jar: CookieJar, mut req: Request, next: Next) -> Response {
    tracing::debug!("{:<12} -- mw_req_session", "MIDDLEWARE");

    tracing::debug!("{:?}", get_auth_cookie(&jar));

    if let Some(token) = get_auth_cookie(&jar)
        && let Ok(TokenData { claims, .. }) = JwtContext::get_from_config().verify(&token)
    {
        let session = Session::from(claims);
        tracing::debug!("Got session {session:?}");
        req.extensions_mut().insert(session);
    }
    next.run(req).await
}
