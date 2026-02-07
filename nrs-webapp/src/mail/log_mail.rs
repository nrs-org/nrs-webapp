use async_trait::async_trait;
use hypertext::Rendered;

use super::Result;
use crate::mail::Mailer;

pub struct LogMailer;

#[async_trait]
impl Mailer for LogMailer {
    /// Logs the email's sender, recipient, subject, and HTML body at info level and always succeeds.
    ///
    /// # Examples
    ///
    /// ```
    /// use hypertext::Rendered;
    /// use futures::executor::block_on;
    /// // Construct the mailer and a rendered HTML body
    /// let mailer = crate::mail::log_mail::LogMailer;
    /// let body = Rendered::new("<p>Hello</p>".to_string());
    /// // Send (logs the message); the call completes successfully
    /// block_on(mailer.send_mail("to@example.com", "from@example.com", "Subject", body)).unwrap();
    /// ```
    async fn send_mail(
        &self,
        to: &str,
        from: &str,
        subject: &str,
        html_body: Rendered<String>,
    ) -> Result<()> {
        tracing::info!(
            "{:<12} -- Sending mail\nFrom: {}\nTo: {}\nSubject: {}\nBody:\n{}",
            "EMAIL",
            from,
            to,
            subject,
            html_body.into_inner()
        );
        Ok(())
    }
}
