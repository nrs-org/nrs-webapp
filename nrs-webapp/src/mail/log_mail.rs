use async_trait::async_trait;
use hypertext::Rendered;

use super::Result;
use crate::mail::Mailer;

pub struct LogMailer;

#[async_trait]
impl Mailer for LogMailer {
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
