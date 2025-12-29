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
    toast_on_page_load,
    toasts::ConstToast,
    validate::auth::{USERNAME_REGEX, validate_password},
};

pub fn router() -> Router<ModelManager> {
    Router::new().route("/", post(submit))
}

#[derive(Deserialize)]
struct LogoffPayload {
    logoff: bool,
}

async fn submit(
    jar: CookieJar,
    WRForm(LogoffPayload { logoff }): WRForm<LogoffPayload>,
) -> Response {
    tracing::debug!("{:<12} -- POST auth::logoff -- logoff: {}", "ROUTE", logoff);

    if !logoff {
        return ().into_response();
    }

    (HxRedirect("/".into()), remove_auth_cookie(jar)).into_response()
}
