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

/// Create a router that mounts the logoff POST handler at the root path.
///
/// # Examples
///
/// ```
/// let r = router();
/// // `r` is ready to be used by the application and has a POST "/" route for logoff.
/// ```
pub fn router() -> Router<ModelManager> {
    Router::new().route("/", post(submit))
}

#[derive(Deserialize)]
struct LogoffPayload {
    logoff: bool,
}

/// Handle a form-based logoff request by optionally clearing the auth cookie and redirecting to the home page.
///
/// If the submitted `logoff` value is `false`, this returns an empty response. If `logoff` is `true`, this removes the authentication cookie from the provided `CookieJar` and returns a redirect to "/".
///
/// # Examples
///
/// ```
/// # use axum_extra::extract::CookieJar;
/// # use axum_extra::extract::Form as WRForm;
/// # use axum::response::Response;
/// # use nrs_webapp::routes::auth::logoff::{submit, LogoffPayload};
/// #
/// # tokio::runtime::Runtime::new().unwrap().block_on(async {
/// let jar = CookieJar::new();
/// let form = WRForm(LogoffPayload { logoff: true });
/// let _resp: Response = submit(jar, form).await;
/// # });
/// ```
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
