use axum::{Router, extract::State, response::IntoResponse, routing::get};
use axum_client_ip::ClientIp;
use axum_extra::{TypedHeader, headers::UserAgent};
use axum_htmx::HxRequest;
use nrs_webapp_frontend::{
    maybe_document,
    views::pages::auth::register::{RegisterScreen, register},
};
use serde::Deserialize;
use validator::Validate;

use crate::{
    Result,
    crypt::password_hash::PasswordHasher,
    extract::{doc_props::DocProps, with_rejection::WRVForm},
    model::{
        ModelManager,
        user::{UserBmc, UserForCreate},
    },
    routes::auth::{
        confirm_mail::redirect_to_confirm_mail_page, mask_email_for_log, mask_username_for_log,
    },
    validate::auth::{USERNAME_REGEX, validate_password},
};

/// Builds a Router<ModelManager> configured with the registration routes.
///
/// The router contains a GET "/" route that serves the registration page.
///
/// # Examples
///
/// ```
/// let r = nrs_webapp::routes::auth::register::router();
/// // mount `r` into your axum application
/// ```
pub fn router() -> Router<ModelManager> {
    Router::new().route("/", get(page).post(submit))
}

/// Render the registration page using the provided document props and HTMX request.
///
/// This handler produces an HTTP response containing the registration page; the response
/// may be a full HTML document or an HTMX fragment depending on the `HxRequest`.
///
/// # Examples
///
/// ```no_run
/// use nrs_webapp::routes::auth::register::page;
/// use nrs_webapp::http::{HxRequest, DocProps};
///
/// // hypothetical usage within an async context — types are provided by the application.
/// # async fn run() {
/// let hx_req: HxRequest = /* obtain or construct HxRequest */;
/// let props = /* construct document props */;
/// let resp = page(hx_req, DocProps(props)).await;
/// # }
/// ```
async fn page(hx_req: HxRequest, DocProps(props): DocProps) -> impl IntoResponse {
    tracing::debug!("{:<12} -- GET auth::register", "ROUTE");
    maybe_document(hx_req, props, register(RegisterScreen::Regular))
}

#[derive(Deserialize, Validate)]
pub(super) struct RegisterPayload {
    #[validate(length(min = 3, max = 20), regex(path=*USERNAME_REGEX))]
    pub username: String,
    #[validate(email, length(max = 100))]
    pub email: String,
    #[validate(length(min = 8, max = 50), custom(function = validate_password))]
    pub password: String,
}

/// Handles POST submissions of the registration form: validates input, hashes the password, creates a new user, and returns a redirect to the email confirmation page on success.
///
/// On success this function persists a new user (username, email, hashed password) and returns a response that redirects the client to the confirmation-mail page. Errors from hashing or persistence are propagated via the returned `Result`.
///
/// # Examples
///
/// ```no_run
/// // This illustrates the intended outcome: submitting valid registration data
/// // results in a redirect response to the confirmation page.
/// // Integration tests should construct an HTTP request and assert the redirect target.
/// ```
async fn submit(
    HxRequest(_hx_req): HxRequest,
    State(mut mm): State<ModelManager>,
    ClientIp(ip_addr): ClientIp,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    WRVForm(RegisterPayload {
        username,
        email,
        password,
    }): WRVForm<RegisterPayload>,
) -> Result<impl IntoResponse> {
    tracing::debug!(
        "{:<12} -- POST auth::register -- username: {}, email: {}",
        "ROUTE",
        mask_username_for_log(&username),
        mask_email_for_log(&email)
    );

    let password_hash = PasswordHasher::get_from_config().encrypt_password(&password)?;

    let _ = UserBmc::create_user(
        &mut mm,
        UserForCreate {
            username: username.clone(),
            email,
            password_hash,
        },
    )
    .await?;

    Ok(redirect_to_confirm_mail_page(
        mm, username, ip_addr, user_agent,
    ))
}
