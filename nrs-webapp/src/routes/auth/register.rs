use std::{
    net::{IpAddr, SocketAddr},
    str::FromStr,
    sync::OnceLock,
};

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
    routes::auth::confirm_mail::redirect_to_confirm_mail_page,
    toast_on_page_load,
    toasts::ConstToast,
    validate::auth::{USERNAME_REGEX, validate_password},
};

pub fn router() -> Router<ModelManager> {
    Router::new().route("/", get(page))
}

async fn page(hx_req: HxRequest, DocProps(props): DocProps) -> impl IntoResponse {
    tracing::debug!("{:<12} -- GET auth::register", "ROUTE");
    maybe_document(hx_req, props, register())
}

#[derive(Deserialize, Validate)]
struct RegisterPayload {
    #[validate(length(min = 3, max = 20), regex(path=*USERNAME_REGEX))]
    username: String,
    #[validate(email)]
    email: String,
    #[validate(length(min = 8), custom(function = validate_password))]
    password: String,
}

async fn submit(
    HxRequest(hx_req): HxRequest,
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
        username,
        email
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
