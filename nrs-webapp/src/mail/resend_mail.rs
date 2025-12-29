use async_trait::async_trait;
use hypertext::Rendered;
use resend_rs::{Resend, types::CreateEmailBaseOptions};

use super::Result;
use crate::mail::Mailer;

pub struct ResendMailer(Resend);

impl ResendMailer {
    /// Create a new ResendMailer backed by a Resend client.
    ///
    /// `api_key` is the Resend API key used to authenticate requests.
    ///
    /// # Examples
    ///
    /// ```
    /// let mailer = crate::mail::resend_mail::ResendMailer::new("sk_test_...");
    /// ```
    pub fn new(api_key: &str) -> Self {
        let resend = Resend::new(api_key);
        Self(resend)
    }
}

#[async_trait]
impl Mailer for ResendMailer {
    /// Sends an HTML email using the Resend client.
    ///
    /// Builds and email with the given sender, single recipient, subject, and HTML body, then dispatches it via the inner Resend client.
    ///
    /// `to` — recipient email address.
    /// `from` — sender email address.
    /// `subject` — message subject.
    /// `html_body` — HTML content wrapped in `Rendered<String>`.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success; propagates any error returned by the Resend client.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use crate::mail::resend_mail::ResendMailer;
    /// # use hypertext::Rendered;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mailer = ResendMailer::new("RE_SEND_API_KEY");
    /// let body = Rendered::from(String::from("<p>Hello</p>"));
    /// mailer.send_mail("recipient@example.com", "sender@example.com", "Greetings", body).await?;
    /// # Ok(()) }
    /// ```
    async fn send_mail(
        &self,
        to: &str,
        from: &str,
        subject: &str,
        html_body: Rendered<String>,
    ) -> Result<()> {
        let email = CreateEmailBaseOptions::new(from, [to], subject)
            .with_html(html_body.as_inner().as_str());
        self.0.emails.send(email).await?;
        Ok(())
    }
}
