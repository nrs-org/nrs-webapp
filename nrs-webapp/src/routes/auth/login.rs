use axum::{
    Router,
    extract::State,
    response::{IntoResponse, Response},
    routing::get,
};
use axum_client_ip::ClientIp;
use axum_extra::{TypedHeader, extract::SignedCookieJar, headers::UserAgent};
use axum_htmx::{HxRedirect, HxRequest};
use nrs_webapp_frontend::{maybe_document, views::pages::auth::login::login};
use serde::Deserialize;
use sqlbindable::Fields;
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;
use validator::Validate;

use crate::{
    Error, Result,
    auth::{self, add_auth_cookie, error::LoginError},
    crypt::{password_hash::PasswordHasher, session_token::SessionToken},
    extract::{doc_props::DocProps, with_rejection::WRVForm},
    model::{ModelManager, user::UserBmc},
    routes::auth::{confirm_mail::redirect_to_confirm_mail_page, mask_username_for_log},
};

/// Create a router that mounts the login page and submission handlers at the root path.
///
/// The returned Router<ModelManager> exposes a GET handler (`page`) and a POST handler (`submit`)
/// mounted at "/".
///
/// # Examples
///
/// ```
/// # use nrs_webapp::routes::auth::login::router;
/// let _router = router();
/// ```
pub fn router() -> Router<ModelManager> {
    Router::new().route("/", get(page).post(submit))
}

/// Render the login page, returning either a full HTML page or an HTMX fragment based on the request.
///
/// Returns an HTTP response containing the login page or an HTMX partial.
///
/// # Examples
///
/// ```
/// # async fn example() {
/// // Typical handler invocation within an async context:
/// let resp = page(hx_req, DocProps(props)).await;
/// // `resp` can be converted into an HTTP response to send to the client.
/// # }
/// ```
async fn page(hx_req: HxRequest, DocProps(props): DocProps) -> impl IntoResponse {
    tracing::debug!("{:<12} -- GET auth::login", "ROUTE");
    maybe_document(hx_req, props, login())
}

#[derive(Deserialize, Validate)]
struct LoginPayload {
    #[validate(length(max = 50))]
    username: String,
    #[validate(length(max = 50))]
    password: String,
}

#[derive(Fields, FromRow)]
struct LoginUser {
    id: Uuid,
    password_hash: String,
    email_verified_at: Option<OffsetDateTime>,
}

/// Handle POST submissions to the login endpoint.
///
/// Authenticates the provided username and password; if credentials are valid and the
/// account's email is verified, issues a session token, attaches an authentication cookie and
/// redirects to the application root. If credentials are valid but the email is not
/// verified, redirects to the email confirmation page. If authentication fails, returns
/// an authentication error.
///
/// # Returns
///
/// - `Ok(Response)` — on success: either a redirect to `/` with an auth cookie (when the
///   user's email is verified) or a redirect to the email confirmation page (when the
///   user's email is not verified).
/// - `Err(Error::Auth(LoginError::InvalidCredentials))` — when the username/password
///   combination is invalid. Other errors from downstream operations (database access,
///   or hashing) are propagated as `Err`.
///
/// # Examples
///
/// ```no_run
/// use axum::response::Response;
/// # async fn run_example() -> Result<(), Box<dyn std::error::Error>> {
/// // Handler wiring and request construction are omitted here; this shows the expected
/// // outcome semantics when calling the handler:
/// // - Valid credentials + verified email -> redirect to `/` with auth cookie.
/// // - Valid credentials + unverified email -> redirect to confirmation page.
/// // - Invalid credentials -> `Err(Error::Auth(LoginError::InvalidCredentials))`.
/// # Ok(()) }
/// ```
async fn submit(
    State(mut mm): State<ModelManager>,
    jar: SignedCookieJar,
    ClientIp(ip_addr): ClientIp,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    WRVForm(LoginPayload { username, password }): WRVForm<LoginPayload>,
) -> Result<Response> {
    tracing::debug!(
        "{:<12} -- POST auth::login -- username: {}",
        "ROUTE",
        mask_username_for_log(&username)
    );

    let user: Option<LoginUser> = UserBmc::get_by_username(&mut mm, &username).await?;
    let password_hash: &str = user
        .as_ref()
        .map(|u| u.password_hash.as_str())
        .unwrap_or_else(|| PasswordHasher::get_from_config().dummy_hash());

    let check_result =
        PasswordHasher::get_from_config().verify_password(&password, password_hash)?;

    let user = match (check_result, user) {
        (true, Some(user)) => user,
        _ => {
            return Err(Error::Auth(auth::Error::Login(
                LoginError::InvalidCredentials,
            )));
        }
    };

    if user.email_verified_at.is_some() {
        Ok((
            HxRedirect("/".into()),
            add_auth_cookie(jar, SessionToken::new(user.id)),
        )
            .into_response())
    } else {
        Ok(redirect_to_confirm_mail_page(mm, username, ip_addr, user_agent).into_response())
    }
}
