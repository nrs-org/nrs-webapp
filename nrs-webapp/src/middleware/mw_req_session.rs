use axum::{extract::Request, middleware::Next, response::Response};
use axum_extra::extract::SignedCookieJar;

use crate::{
    auth::{get_auth_cookie, session::Session},
    crypt::session_token::SessionToken,
};

/// Middleware that attaches an authenticated Session to request extensions when a valid auth cookie is present.
///
/// If an auth cookie exists and the session token parses and validates successfully, a `Session` constructed from the token
/// is inserted into the request's extensions. The request is forwarded to the next handler regardless of
/// whether a session was inserted; the middleware returns the response produced by the next handler.
///
/// # Examples
///
/// ```
/// use axum::{Router, routing::get};
/// // Mount the middleware onto a router route or the entire router.
/// let app = Router::new()
///     .route("/", get(|| async { "ok" }))
///     .layer(axum::middleware::from_fn(crate::middleware::mw_req_session));
/// ```
pub async fn mw_req_session(jar: SignedCookieJar, mut req: Request, next: Next) -> Response {
    tracing::debug!("{:<12} -- mw_req_session", "MIDDLEWARE");

    tracing::debug!("{:?}", get_auth_cookie(&jar));

    if let Some(token) = get_auth_cookie(&jar)
        && let Ok(token) = token.parse::<SessionToken>()
        && let Ok(user_id) = token.validate()
    {
        let session = Session::new(user_id);
        tracing::debug!("Got session {session:?}");
        req.extensions_mut().insert(session);
    }
    next.run(req).await
}
