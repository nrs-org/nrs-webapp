use async_trait::async_trait;
use hypertext::{Renderable, Rendered};
use resend_rs::{
    Resend,
    types::{CreateEmailBaseOptions, Email},
};

use super::Result;
use crate::{config::AppConfig, mail::Mailer};

pub struct ResendMailer(Resend);

impl ResendMailer {
    pub fn new(api_key: &str) -> Self {
        let resend = Resend::new(api_key);
        Self(resend)
    }
}

#[async_trait]
impl Mailer for ResendMailer {
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
