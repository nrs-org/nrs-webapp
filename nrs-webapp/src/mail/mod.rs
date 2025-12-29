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

/// Returns a shared, static mailer implementation chosen from application configuration.
///
/// The returned reference points to a singleton `Mailer` instance: if `AppConfig::RESEND_API_KEY`
/// is set, a `ResendMailer` is used; otherwise a `LogMailer` is used.
///
/// # Examples
///
/// ```
/// let m1 = get_mailer();
/// let m2 = get_mailer();
/// // both calls return the same static instance
/// assert!(std::ptr::eq(m1, m2));
/// ```
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

/// Get the configured support email address used as the sender for account-related messages.
///
/// If the application configuration does not specify a support address, this returns
/// "accounts@nrs.dev".
///
/// # Examples
///
/// ```
/// let support = email_account_support();
/// assert!(support.contains('@'));
/// ```
fn email_account_support() -> &'static str {
    AppConfig::get()
        .EMAIL_ACCOUNT_SUPPORT
        .as_deref()
        .unwrap_or("accounts@nrs.dev")
}

/// Sends an email verification message containing a confirmation link to the specified user email.
///
/// The message includes a generated confirmation URL that embeds the provided token and uses the configured
/// support address as the sender.
///
/// # Examples
///
/// ```no_run
/// # async fn run() -> nrs_webapp::Result<()> {
/// // Construct or obtain a `Token` appropriate for email confirmation.
/// let token = /* Token for confirmation */ unimplemented!();
/// nrs_webapp::mail::send_email_verification_mail("user@example.com", "alice", &token).await?;
/// # Ok(())
/// # }
/// ```
///
/// @returns `Ok(())` on success, `Err` on failure.
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

/// Sends a password-reset email to the specified user containing a link with the provided token.
///
/// The message uses the subject "nrs-webapp - Password Reset Request" and is sent from the configured support address.
///
/// # Errors
///
/// Returns an error if the mailer fails to send the message.
///
/// # Examples
///
/// ```no_run
/// use nrs_webapp::mail::send_password_reset_mail;
/// use nrs_webapp::token::Token;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let token = Token::from("example-token");
/// send_password_reset_mail("user@example.com", "alice", &token).await?;
/// # Ok(())
/// # }
/// ```
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