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
use axum_extra::{TypedHeader, extract::CookieJar, headers::UserAgent};
use axum_htmx::{HxPushUrl, HxRedirect, HxRefresh, HxRequest, HxTarget};
use base64::{Engine, prelude::BASE64_URL_SAFE};
use futures::FutureExt as _;
use governor::{DefaultKeyedRateLimiter, Quota, RateLimiter};
use nonzero_ext::nonzero;
use nrs_webapp_frontend::{
    maybe_document,
    views::pages::auth::{
        confirm_email::confirm_mail, forgot_pass::forgot_pass, login::login, register::register,
    },
};
use serde::Deserialize;
use sqlbindable::Fields;
use sqlx::prelude::FromRow;
use time::OffsetDateTime;
use validator::Validate;

use crate::{
    Error, Result,
    auth::{self, add_auth_cookie, error::LoginError, remove_auth_cookie},
    config::AppConfig,
    crypt::{
        jwt::JwtContext,
        password_hash::PasswordHasher,
        token::{Token, TokenHasher},
    },
    extract::{
        doc_props::DocProps,
        with_rejection::{WRForm, WRQuery, WRVForm},
    },
    mail::{get_mailer, send_email_verification_mail},
    model::{
        self, ModelManager,
        token::{TokenPurpose, UserOneTimeTokenBmc, UserOneTimeTokenCreateReq},
        user::{UserBmc, UserForCreate},
    },
    toast_on_page_load,
    toasts::ConstToast,
    validate::auth::{USERNAME_REGEX, validate_password},
};

pub fn router() -> Router<ModelManager> {
    Router::new()
        .route("/", get(confirm_page).post(resend_mail))
        .route("/confirm", get(confirm_submit))
}

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

async fn confirm_page(
    hx_req: HxRequest,
    State(mm): State<ModelManager>,
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

async fn confirm_submit(
    State(mut mm): State<ModelManager>,
    ClientIp(ip_addr): ClientIp,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
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

    UserBmc::mark_email_verified(&mut tx, &user_id)
        .always_send()
        .await?;
    tx.commit().await?;

    let url = format!(
        "/auth/login?{}",
        toast_on_page_load!(ConstToast::LoginAgainAfterEmailVerification)
    );
    Ok((HxPushUrl("/auth/login".into()), Redirect::to(&url)))
}

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
    id: String,
    email: String,
    email_verified_at: Option<OffsetDateTime>,
}

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
