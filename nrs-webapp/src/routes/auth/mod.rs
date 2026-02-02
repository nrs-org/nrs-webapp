mod confirm_mail;
mod forgot_password;
mod login;
mod logoff;
mod register;

use axum::Router;

use crate::model::ModelManager;

/// Constructs a Router exposing all authentication-related endpoints.
///
/// The returned `Router<ModelManager>` nests the sub-routers for login, register, logoff, email confirmation, and password recovery
/// under "/login", "/register", "/logoff", "/confirmmail", and "/forgotpass" respectively. State is inherited from the parent router.
///
/// # Examples
///
/// ```no_run
/// use nrs_webapp::routes::auth::router;
/// let auth_router = router();
/// ```
pub fn router() -> Router<ModelManager> {
    Router::new()
        .nest("/login", login::router())
        .nest("/register", register::router())
        .nest("/logoff", logoff::router())
        .nest("/confirmmail", confirm_mail::router())
        .nest("/forgotpass", forgot_password::router())
}

pub(crate) fn mask_email_for_log(email: &str) -> String {
    if let Some((local, domain)) = email.split_once('@') {
        let first = local.chars().next().unwrap_or('*');
        format!("{}***@{}", first, domain)
    } else {
        "<redacted-email>".to_string()
    }
}
