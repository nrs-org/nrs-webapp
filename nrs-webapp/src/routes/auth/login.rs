use axum::{
    Router,
    extract::State,
    response::{IntoResponse, Response},
    routing::get,
};
use axum_client_ip::ClientIp;
use axum_extra::{TypedHeader, extract::CookieJar, headers::UserAgent};
use axum_htmx::{HxRedirect, HxRequest};
use nrs_webapp_frontend::{maybe_document, views::pages::auth::login::login};
use serde::Deserialize;
use sqlbindable::Fields;
use sqlx::FromRow;
use time::OffsetDateTime;
use validator::Validate;

use crate::{
    Error, Result,
    auth::{self, add_auth_cookie, error::LoginError},
    crypt::{jwt::JwtContext, password_hash::PasswordHasher},
    extract::{doc_props::DocProps, with_rejection::WRForm},
    model::{ModelManager, user::UserBmc},
    routes::auth::confirm_mail::redirect_to_confirm_mail_page,
};

pub fn router() -> Router<ModelManager> {
    Router::new().route("/", get(page).post(submit))
}

async fn page(hx_req: HxRequest, DocProps(props): DocProps) -> impl IntoResponse {
    tracing::debug!("{:<12} -- GET auth::login", "ROUTE");
    maybe_document(hx_req, props, login())
}

#[derive(Deserialize, Validate)]
struct LoginPayload {
    username: String,
    password: String,
}

#[derive(Fields, FromRow)]
struct LoginUser {
    id: String,
    password_hash: String,
    email_verified_at: Option<OffsetDateTime>,
}

async fn submit(
    hx_req: HxRequest,
    State(mut mm): State<ModelManager>,
    jar: CookieJar,
    ClientIp(ip_addr): ClientIp,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    WRForm(LoginPayload { username, password }): WRForm<LoginPayload>,
) -> Result<Response> {
    tracing::debug!(
        "{:<12} -- POST auth::login -- username: {}",
        "ROUTE",
        username
    );

    let user: Option<LoginUser> = UserBmc::get_by_username(&mut mm, &username).await?;
    let password_hash: &str = user
        .as_ref()
        .map(|u| u.password_hash.as_str())
        .unwrap_or_else(|| PasswordHasher::get_from_config().dummy_hash());

    let check_result =
        PasswordHasher::get_from_config().verify_password(&password, password_hash)?;

    let user = match (check_result, user) {
        (true, Some(user)) => user,
        _ => {
            return Err(Error::Auth(auth::Error::Login(
                LoginError::InvalidCredentials,
            )));
        }
    };

    if user.email_verified_at.is_some() {
        let jwt = JwtContext::get_from_config();
        let claims = jwt.generate_claims(user.id);
        let token = jwt.sign(&claims)?;

        Ok((HxRedirect("/".into()), add_auth_cookie(jar, token)).into_response())
    } else {
        Ok(redirect_to_confirm_mail_page(
            mm, username, ip_addr, user_agent,
        ))
    }
}
