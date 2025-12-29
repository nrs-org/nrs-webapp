use axum::{
    Router,
    response::{IntoResponse, Response},
    routing::post,
};
use axum_extra::extract::CookieJar;
use axum_htmx::HxRedirect;
use serde::Deserialize;

use crate::{auth::remove_auth_cookie, extract::with_rejection::WRForm, model::ModelManager};

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
