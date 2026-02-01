use std::{net::IpAddr, str::FromStr, sync::OnceLock};

use always_send::FutureExt;
use axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    routing::get,
};
use axum_client_ip::ClientIp;
use axum_extra::{TypedHeader, headers::UserAgent};
use axum_htmx::{HxPushUrl, HxRedirect, HxRequest};
use governor::{DefaultKeyedRateLimiter, Quota, RateLimiter};
use nonzero_ext::nonzero;
use nrs_webapp_frontend::{maybe_document, views::pages::auth::confirm_email::confirm_mail};
use serde::Deserialize;
use sqlbindable::Fields;
use sqlx::prelude::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    Error, Result,
    config::AppConfig,
    crypt::token::{Token, TokenHasher},
    extract::{
        doc_props::DocProps,
        with_rejection::{WRForm, WRQuery},
    },
    mail::send_email_verification_mail,
    model::{
        ModelManager,
        token::{TokenPurpose, UserOneTimeTokenBmc, UserOneTimeTokenCreateReq},
        user::UserBmc,
    },
    toast_on_page_load,
    toasts::ConstToast,
};

/// Creates the HTTP router for the confirm-email endpoints.
///
/// The router exposes:
/// - GET "/" -> renders the confirm-mail page
/// - POST "/" -> triggers resending the confirmation email
/// - GET "/confirm" -> consumes an email verification token and finalizes confirmation
///
/// # Examples
///
/// ```
/// let _router = nrs_webapp::routes::auth::confirm_mail::router();
/// ```
pub fn router() -> Router<ModelManager> {
    Router::new()
        .route("/", get(confirm_page).post(resend_mail))
        .route("/confirm", get(confirm_submit))
}

/// Redirects the client to the confirm-mail page for a username and starts sending the confirmation email in the background.
///
/// The client receives an HTMX redirect with HTTP status 204 No Content that points to `/auth/confirmmail?username=<encoded>`.
///
/// # Examples
///
/// ```
/// use std::net::IpAddr;
/// use axum::response::Response;
/// use http::StatusCode;
/// // Construct or obtain a ModelManager and UserAgent in real usage.
/// let mm = /* ModelManager */ unimplemented!();
/// let username = "alice".to_string();
/// let ip_addr: IpAddr = "127.0.0.1".parse().unwrap();
/// let user_agent = /* UserAgent */ unimplemented!();
///
/// let resp: Response = redirect_to_confirm_mail_page(mm, username, ip_addr, user_agent);
/// assert_eq!(resp.status(), StatusCode::NO_CONTENT);
/// ```
pub fn redirect_to_confirm_mail_page(
    mm: ModelManager,
    username: String,
    ip_addr: IpAddr,
    user_agent: UserAgent,
) -> Response {
    let url = format!(
        "/auth/confirmmail?username={}",
        urlencoding::encode(&username)
    );

    tokio::spawn(send_confirm_mail(mm, username, ip_addr, user_agent));
    (HxRedirect(url), StatusCode::NO_CONTENT).into_response()
}

#[derive(Deserialize)]
struct ConfirmPagePayload {
    username: String,
}

/// Render the email confirmation page for the given username, returning an HTTP response
/// appropriate for either a full-page request or an HTMX request.
///
/// The handler uses the provided ModelManager and documentation properties to produce the
/// confirmation page; when the request is an HTMX request it will return the corresponding
/// HTMX response.
///
/// # Examples
///
/// ```
/// use axum::{routing::get, Router};
///
/// // mount the handler at the root of a router
/// let app = Router::new().route("/", get(crate::routes::auth::confirm_mail::confirm_page));
/// ```
async fn confirm_page(
    hx_req: HxRequest,
    State(_mm): State<ModelManager>,
    WRQuery(ConfirmPagePayload { username }): WRQuery<ConfirmPagePayload>,
    DocProps(props): DocProps,
) -> impl IntoResponse {
    tracing::debug!("{:<12} -- GET auth::confirm_mail", "ROUTE");

    maybe_document(hx_req, props, confirm_mail(username))
}

#[derive(Deserialize)]
struct ConfirmSubmitPayload {
    token: String,
}

/// Handle a POST of an email confirmation token, verify the token, mark the user's email as verified, and redirect to the login page with a toast.
///
/// On success this function consumes the provided one-time email verification token, marks the associated user's email as verified within a database transaction, commits the transaction, and returns a response that pushes an HTMX URL and redirects the client to the login page with a toast message.
///
/// # Returns
/// `Ok` containing a combined HTMX push and an HTTP redirect to the login page on success; an error if token parsing, token verification/consumption, database operations, or transaction commit fail.
///
/// # Examples
///
/// ```
/// // Example usage (handler functions are normally invoked by the web framework):
/// // let res = confirm_submit(state, client_ip, user_agent_header, WRQuery(ConfirmSubmitPayload { token: "..." .into() })).await;
/// // assert!(res.is_ok());
/// ```
async fn confirm_submit(
    State(mm): State<ModelManager>,
    ClientIp(_ip_addr): ClientIp,
    TypedHeader(_user_agent): TypedHeader<UserAgent>,
    WRQuery(ConfirmSubmitPayload { token }): WRQuery<ConfirmSubmitPayload>,
) -> Result<impl IntoResponse> {
    tracing::debug!("{:<12} -- POST auth::confirm_submit", "ROUTE");

    let token = Token::from_str(&token)?;

    let mut tx = mm.tx().await?;
    let user_id = UserOneTimeTokenBmc::check_and_consume_token(
        &mut tx,
        &TokenHasher::get_from_config().hash(&token),
        TokenPurpose::EmailVerification,
    )
    .always_send()
    .await?;

    UserBmc::mark_email_verified(&mut tx, user_id)
        .always_send()
        .await?;
    tx.commit().await?;

    let url = format!(
        "/auth/login?{}",
        toast_on_page_load!(ConstToast::LoginAgainAfterEmailVerification)
    );
    Ok((HxPushUrl("/auth/login".into()), Redirect::to(&url)))
}

/// Triggers sending a confirmation email for the given username in the background and returns an HTMX push response with HTTP 204.
///
/// This handler spawns an asynchronous task to (re)send the confirmation email for `username` and immediately responds with an HTMX push payload of `"false"` and status `204 No Content`.
///
/// # Examples
///
/// ```
/// # use std::net::IpAddr;
/// # use tokio;
/// # async fn example(mm: crate::model::ModelManager, username: String, ip_addr: IpAddr, ua: crate::http::UserAgent) {
/// // Equivalent effect: spawn the background email send task.
/// tokio::spawn(crate::routes::auth::confirm_mail::send_confirm_mail(mm, username, ip_addr, ua));
/// # }
/// ```
async fn resend_mail(
    State(mm): State<ModelManager>,
    ClientIp(ip_addr): ClientIp,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    WRForm(ConfirmPagePayload { username }): WRForm<ConfirmPagePayload>,
) -> impl IntoResponse {
    tracing::debug!("{:<12} -- POST auth::confirm_mail_resend", "ROUTE");

    tokio::spawn(send_confirm_mail(mm, username, ip_addr, user_agent));
    (HxPushUrl("false".into()), StatusCode::NO_CONTENT)
}

/// Sends a confirmation email for the given username and logs an error if delivery fails.
///
/// This function triggers the internal email-sending workflow and suppresses any error by
/// logging it; it does not return error information to the caller.
///
/// # Parameters
///
/// - `mm`: Application model manager used to access persistence and configuration.
/// - `username`: Username identifying the account to which the confirmation email should be sent.
/// - `ip_addr`: Client IP address associated with the request that triggered the email.
/// - `user_agent`: User agent string associated with the request.
///
/// # Examples
///
/// ```no_run
/// use std::net::IpAddr;
/// // Assume `mm`, `username`, and `user_agent` are available in scope.
/// // let mm = /* ModelManager */ ;
/// // let username = "alice".to_string();
/// // let ip_addr: IpAddr = "127.0.0.1".parse().unwrap();
/// // let user_agent = /* UserAgent */ ;
///
/// // Spawn the async send without awaiting its result:
/// tokio::spawn(send_confirm_mail(mm, username, ip_addr, user_agent));
/// ```
async fn send_confirm_mail(
    mm: ModelManager,
    username: String,
    ip_addr: IpAddr,
    user_agent: UserAgent,
) {
    if let Err(err) = send_confirm_email_inner(mm, username, ip_addr, user_agent.to_string()).await
    {
        tracing::error!(
            "{:<12} -- send_confirm_mail -- Error sending confirm email: {}",
            "FOR-DEV-ONLY",
            err
        );
    }
}

#[derive(Debug, FromRow, Fields)]
struct UserIdEmail {
    id: Uuid,
    email: String,
    email_verified_at: Option<OffsetDateTime>,
}

/// Sends an email verification token to the given username if that user exists and their email is not yet verified.
///
/// This function enforces a per-username rate limit, generates and stores a one-time verification token with an expiry,
/// and attempts to deliver a verification email to the user's address. If no unverified user is found for the given
/// username the function completes successfully without sending mail.
///
/// # Errors
///
/// Returns an error if rate limiting prevents the request, token generation or hashing fails, database operations fail,
/// or email delivery fails.
///
/// # Examples
///
/// ```no_run
/// use std::net::IpAddr;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Obtain a ModelManager `mm`, e.g. from your application state.
/// let mm = /* ModelManager instance */ unimplemented!();
/// let username = "alice".to_string();
/// let ip_addr: IpAddr = "127.0.0.1".parse().unwrap();
/// let user_agent = "example-agent".to_string();
///
/// // Attempt to send a confirmation email (may return an application Error).
/// let _ = nrs_webapp::routes::auth::confirm_mail::send_confirm_email_inner(mm, username, ip_addr, user_agent).await?;
/// # Ok(()) }
/// ```
async fn send_confirm_email_inner(
    mm: ModelManager,
    username: String,
    ip_addr: IpAddr,
    user_agent: String,
) -> Result<()> {
    static RATE_LIMITER: OnceLock<DefaultKeyedRateLimiter<String>> = OnceLock::new();

    tracing::debug!(
        "{:<12} -- send_confirm_email -- username: {}",
        "FOR-DEV-ONLY",
        username
    );

    RATE_LIMITER
        .get_or_init(|| RateLimiter::keyed(Quota::per_minute(nonzero!(1u32))))
        .check_key(&username)
        .map_err(|_| Error::RateLimitExceeded {
            service: "confirm-email",
        })?;

    let confirm_token = Token::generate()?;
    let confirm_token_hash = TokenHasher::get_from_config().hash(&confirm_token);

    let mut tx = mm.tx().await?;

    // NOTE: this triggers a compiler bug without .always_send()
    // (.boxed() works too but that would harm perf for nothing)
    // see: https://github.com/rust-lang/rust/issues/96645
    if let Some(UserIdEmail {
        id,
        email,
        email_verified_at: None,
    }) = UserBmc::get_by_username(&mut tx, &username)
        .always_send()
        .await?
    {
        UserOneTimeTokenBmc::create_token(
            &mut tx,
            UserOneTimeTokenCreateReq {
                user_id: id,
                purpose: TokenPurpose::EmailVerification,
                token_hash: confirm_token_hash,
                expires_at: OffsetDateTime::now_utc()
                    + AppConfig::get().email_verification_expiry_duration(),
                request_ip: Some(ip_addr.to_string()),
                user_agent: Some(user_agent),
            },
        )
        .always_send()
        .await?;

        tx.commit().await?;

        send_email_verification_mail(&email, &username, &confirm_token).await?;
    } else {
        tracing::debug!(
            "{:<12} -- send_confirm_email -- No unverified user found with username: {}",
            "FOR-DEV-ONLY",
            username
        );
    }

    Ok(())
}
