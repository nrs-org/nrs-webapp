use std::{net::IpAddr, str::FromStr, sync::OnceLock};

use always_send::FutureExt;
use axum::{
    Form, Router,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
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
        confirm_email::confirm_mail,
        forgot_pass::{forgot_pass, forgot_pass_sent, reset_pass},
        login::login,
        register::register,
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
    mail::{get_mailer, send_email_verification_mail, send_password_reset_mail},
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
        .route("/", get(email_page).post(email_submit))
        .route("/reset", get(reset_page).post(reset_submit))
}

async fn email_page(hx_req: HxRequest, DocProps(props): DocProps) -> impl IntoResponse {
    tracing::debug!("{:<12} -- GET auth::forgot_pass", "ROUTE");
    maybe_document(hx_req, props, forgot_pass())
}

#[derive(Deserialize, Validate)]
struct ResetPasswordQuery {
    token: String,
}

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

async fn email_submit(
    DocProps(props): DocProps,
    State(mut mm): State<ModelManager>,
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
    #[validate(custom(function = validate_password))]
    password: String,
}

async fn reset_submit(
    DocProps(props): DocProps,
    State(mut mm): State<ModelManager>,
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
    id: String,
    username: String,
    email_verified_at: Option<OffsetDateTime>,
}

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
