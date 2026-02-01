use std::{net::IpAddr, str::FromStr, sync::OnceLock};

use always_send::FutureExt;
use axum::{
    Router,
    extract::State,
    response::{IntoResponse, Redirect},
    routing::get,
};
use axum_client_ip::ClientIp;
use axum_extra::{TypedHeader, headers::UserAgent};
use axum_htmx::{HxPushUrl, HxRequest};
use governor::{DefaultKeyedRateLimiter, Quota, RateLimiter};
use nonzero_ext::nonzero;
use nrs_webapp_frontend::{
    maybe_document,
    views::pages::auth::forgot_pass::{forgot_pass, forgot_pass_sent, reset_pass},
};
use serde::Deserialize;
use sqlbindable::Fields;
use sqlx::prelude::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;
use validator::Validate;

use crate::{
    Error, Result,
    config::AppConfig,
    crypt::{
        password_hash::PasswordHasher,
        token::{Token, TokenHasher},
    },
    extract::{
        doc_props::DocProps,
        with_rejection::{WRQuery, WRVForm},
    },
    mail::send_password_reset_mail,
    model::{
        ModelManager,
        token::{TokenPurpose, UserOneTimeTokenBmc, UserOneTimeTokenCreateReq},
        user::UserBmc,
    },
    toast_on_page_load,
    toasts::ConstToast,
    validate::auth::validate_password,
};

/// Creates a Router configured with the forgot-password endpoints.
///
/// The router mounts:
/// - `GET /` and `POST /` for requesting a password reset (email submission).
/// - `GET /reset` and `POST /reset` for rendering and submitting a password reset using a token.
///
/// # Examples
///
/// ```rust,no_run
/// let router = router();
/// // mount into an Axum application, e.g. `Router::new().merge(router)`
/// ```
pub fn router() -> Router<ModelManager> {
    Router::new()
        .route("/", get(email_page).post(email_submit))
        .route("/reset", get(reset_page).post(reset_submit))
}

/// Render the forgot-password page, adapting output for HTMX requests.
///
/// Uses the provided document properties and HTMX request context to produce the
/// appropriate HTML response (full page or HTMX fragment).
///
/// # Returns
///
/// A response rendering the forgot-password page; HTMX requests receive an HTMX-compatible fragment.
///
/// # Examples
///
/// ```no_run
/// // Called by the router; shown here for illustration only.
/// // let resp = email_page(hx_req, DocProps(props)).await;
/// ```
async fn email_page(hx_req: HxRequest, DocProps(props): DocProps) -> impl IntoResponse {
    tracing::debug!("{:<12} -- GET auth::forgot_pass", "ROUTE");
    maybe_document(hx_req, props, forgot_pass())
}

#[derive(Deserialize, Validate)]
struct ResetPasswordQuery {
    token: String,
}

/// Render the password reset page for a given reset token.
///
/// Embeds the provided token into the reset-password page and returns an HTMX-aware response
/// suitable for full or partial (HTMX) rendering.
///
/// # Examples
///
/// ```rust,no_run
/// // render page for token "abc123"
/// let resp = reset_page(hx_req, DocProps(props), WRQuery(ResetPasswordQuery { token: "abc123".into() })).await;
/// ```
async fn reset_page(
    hx_req: HxRequest,
    DocProps(props): DocProps,
    WRQuery(ResetPasswordQuery { token }): WRQuery<ResetPasswordQuery>,
) -> impl IntoResponse {
    tracing::debug!("{:<12} -- GET auth::forgot_pass::reset", "ROUTE");
    maybe_document(hx_req, props, reset_pass(token))
}

#[derive(Deserialize, Validate)]
struct EmailSubmitPayload {
    #[validate(email)]
    email: String,
}

/// Handles the forgot-password form submission by enqueueing a background task to send a password-reset link
/// and returning the "forgot password sent" page.
///
/// The handler extracts the submitted email, client IP, user agent, and a model manager, spawns an
/// asynchronous task to send the reset link, and immediately responds with the confirmation page.
///
/// # Returns
///
/// The HTTP response for the forgot-password confirmation page.
///
/// # Examples
///
/// ```
/// use axum::routing::post;
/// // assuming `email_submit` and `router` are in scope
/// let app = router().route("/forgot", post(email_submit));
/// ```
async fn email_submit(
    DocProps(props): DocProps,
    State(mm): State<ModelManager>,
    ClientIp(ip_addr): ClientIp,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    WRVForm(EmailSubmitPayload { email }): WRVForm<EmailSubmitPayload>,
) -> impl IntoResponse {
    tracing::debug!("{:<12} -- POST auth::forgot_pass", "ROUTE");

    tokio::spawn(send_reset_password_link(mm, email, ip_addr, user_agent));
    maybe_document(HxRequest(true), props, forgot_pass_sent())
}

#[derive(Deserialize, Validate)]
struct ResetPasswordSubmitPayload {
    token: String,
    #[validate(length(min = 8, max = 50), custom(function = validate_password))]
    password: String,
}

/// Handle a password-reset form submission: validate and consume the reset token, update the user's password inside a transaction, commit, and redirect the client to the login page with a toast indicating they must sign in again.
///
/// On success, returns a response that pushes the login URL to the client and redirects to the login page with a toast informing the user to log in again.
///
/// # Examples
///
/// ```
/// # use axum::response::Redirect;
/// # use some_crate::HxPushUrl;
/// // The handler returns a push + redirect that navigates the client to the login page
/// let url = format!("/auth/login?{}", /* toast_on_page_load!(ConstToast::LoginAgainAfterPasswordReset) */ "toast=login_again");
/// let resp = (HxPushUrl("/auth/login".into()), Redirect::to(&url));
/// ```
async fn reset_submit(
    State(mm): State<ModelManager>,
    WRVForm(ResetPasswordSubmitPayload { token, password }): WRVForm<ResetPasswordSubmitPayload>,
) -> Result<impl IntoResponse> {
    tracing::debug!("{:<12} -- POST auth::forgot_pass::reset", "ROUTE");

    let mut tx = mm.tx().await?;

    let token_hash = TokenHasher::get_from_config().hash(&Token::from_str(&token)?);
    let user_id = UserOneTimeTokenBmc::check_and_consume_token(
        &mut tx,
        &token_hash,
        TokenPurpose::PasswordReset,
    )
    .always_send()
    .await?;

    let password_hash = PasswordHasher::get_from_config().encrypt_password(&password)?;
    UserBmc::reset_password(&mut tx, user_id, password_hash)
        .always_send()
        .await?;

    tx.commit().await?;

    let url = format!(
        "/auth/login?{}",
        toast_on_page_load!(ConstToast::LoginAgainAfterPasswordReset)
    );
    Ok((HxPushUrl("/auth/login".into()), Redirect::to(&url)))
}

/// Attempts to send a password-reset link for the specified username and logs an error if the operation fails.
///
/// # Examples
///
/// ```no_run
/// use std::net::IpAddr;
/// # async fn example() {
/// let mm = /* obtain ModelManager */ unimplemented!();
/// let ip: IpAddr = "127.0.0.1".parse().unwrap();
/// let ua = /* construct a UserAgent */ unimplemented!();
/// send_reset_password_link(mm, "alice@example.com".to_string(), ip, ua).await;
/// # }
/// ```
async fn send_reset_password_link(
    mm: ModelManager,
    username: String,
    ip_addr: IpAddr,
    user_agent: UserAgent,
) {
    if let Err(err) =
        send_reset_password_link_inner(mm, username, ip_addr, user_agent.to_string()).await
    {
        tracing::error!(
            "{:<12} -- send_reset_password_link -- Error sending reset password link: {}",
            "FOR-DEV-ONLY",
            err
        );
    }
}

#[derive(Debug, FromRow, Fields)]
struct UserIdNameEmailVerifiedAt {
    id: Uuid,
    username: String,
    email_verified_at: Option<OffsetDateTime>,
}

/// Send a password-reset link to a verified user identified by `email`.
///
/// This function enforces a per-email rate limit (5 requests per minute), generates a one-time
/// password-reset token, stores a hashed token record with metadata in the database, commits the
/// transaction, and sends the reset email. If no user with a verified email is found, no email is
/// sent and the function returns `Ok(())`.
///
/// # Errors
///
/// Returns an error if the rate limit is exceeded (`Error::RateLimitExceeded`) or if token
/// generation, database operations, or email delivery fail.
///
/// # Parameters
///
/// - `mm`: Model manager used to start a database transaction and perform data operations.
/// - `email`: Recipient email address to look up and (if verified) to send the reset link to.
/// - `ip_addr`: Request IP address to record with the one-time token metadata.
/// - `user_agent`: Request user-agent string to record with the one-time token metadata.
///
/// # Returns
///
/// `Ok(())` on success; `Err(...)` on failure.
///
/// # Examples
///
/// ```no_run
/// use std::net::IpAddr;
/// # async fn doc_example(mm: crate::ModelManager) -> anyhow::Result<()> {
/// let email = "user@example.com".to_string();
/// let ip_addr: IpAddr = "127.0.0.1".parse().unwrap();
/// let user_agent = "example-agent/1.0".to_string();
///
/// // Call from an async context
/// send_reset_password_link_inner(mm, email, ip_addr, user_agent).await?;
/// # Ok(())
/// # }
/// ```
async fn send_reset_password_link_inner(
    mm: ModelManager,
    email: String,
    ip_addr: IpAddr,
    user_agent: String,
) -> Result<()> {
    static RATE_LIMITER: OnceLock<DefaultKeyedRateLimiter<String>> = OnceLock::new();

    tracing::debug!(
        "{:<12} -- send_reset_password_link -- email: {}",
        "FOR-DEV-ONLY",
        email
    );

    RATE_LIMITER
        .get_or_init(|| RateLimiter::keyed(Quota::per_minute(nonzero!(5u32))))
        .check_key(&email)
        .map_err(|_| Error::RateLimitExceeded {
            service: "password-reset",
        })?;

    let confirm_token = Token::generate()?;
    let confirm_token_hash = TokenHasher::get_from_config().hash(&confirm_token);

    let mut tx = mm.tx().await?;

    if let Some(UserIdNameEmailVerifiedAt {
        id,
        username,
        email_verified_at: Some(_),
    }) = UserBmc::get_by_email(&mut tx, &email).always_send().await?
    {
        UserOneTimeTokenBmc::create_token(
            &mut tx,
            UserOneTimeTokenCreateReq {
                user_id: id,
                purpose: TokenPurpose::PasswordReset,
                token_hash: confirm_token_hash,
                expires_at: OffsetDateTime::now_utc()
                    + AppConfig::get().password_reset_expiry_duration(),
                request_ip: Some(ip_addr.to_string()),
                user_agent: Some(user_agent),
            },
        )
        .always_send()
        .await?;

        tx.commit().await?;

        send_password_reset_mail(&email, &username, &confirm_token).await?;
    } else {
        tracing::debug!(
            "{:<12} -- send_reset_password_link -- No verified user found with email: {}",
            "FOR-DEV-ONLY",
            email
        );
    }

    Ok(())
}
