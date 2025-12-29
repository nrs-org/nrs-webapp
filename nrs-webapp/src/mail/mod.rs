mod error;
mod log_mail;
mod resend_mail;
mod web_mail;

use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use base64::{Engine, prelude::BASE64_URL_SAFE};
pub use error::{Error, Result};
use hypertext::{Renderable, Rendered};
use nrs_webapp_frontend::views::email::{
    email_verify::email_verify, password_reset::password_reset,
};

use crate::{
    config::AppConfig,
    crypt::token::Token,
    mail::{log_mail::LogMailer, resend_mail::ResendMailer},
};

#[async_trait]
pub trait Mailer: Send + Sync {
    async fn send_mail(
        &self,
        to: &str,
        from: &str,
        subject: &str,
        html_body: Rendered<String>,
    ) -> Result<()>;
}

pub fn get_mailer() -> &'static dyn Mailer {
    static MAILER: OnceLock<Arc<dyn Mailer>> = OnceLock::new();
    MAILER
        .get_or_init(|| {
            if let Some(resend_api_key) = AppConfig::get().RESEND_API_KEY.as_ref() {
                tracing::info!("{:<12} -- Using Resend mailer", "MAILER-IMPL");
                Arc::new(ResendMailer::new(resend_api_key.as_str()))
            } else {
                tracing::info!("{:<12} -- Using Log mailer", "MAILER-IMPL");
                Arc::new(LogMailer)
            }
        })
        .as_ref()
}

fn email_account_support() -> &'static str {
    AppConfig::get()
        .EMAIL_ACCOUNT_SUPPORT
        .as_deref()
        .unwrap_or("accounts@nrs.dev")
}

pub async fn send_email_verification_mail(
    user_email: &str,
    username: &str,
    token: &Token,
) -> Result<()> {
    tracing::debug!(
        "{:<12} -- Sending email verification mail to {}",
        "MAILER",
        user_email
    );

    let subject = "nrs-webapp - Please verify your email address";
    let href = format!("http://localhost:3621/auth/confirmmail/confirm?token={token}");

    let body = email_verify(username, &href);

    get_mailer()
        .send_mail(user_email, email_account_support(), subject, body.render())
        .await?;

    Ok(())
}

pub async fn send_password_reset_mail(
    user_email: &str,
    username: &str,
    token: &Token,
) -> Result<()> {
    tracing::debug!(
        "{:<12} -- Sending password reset mail to {}",
        "MAILER",
        user_email
    );

    let subject = "nrs-webapp - Password Reset Request";
    let href = format!("http://localhost:3621/auth/forgotpass/reset?token={token}");

    let body = password_reset(username, &href);

    get_mailer()
        .send_mail(user_email, email_account_support(), subject, body.render())
        .await?;

    Ok(())
}
