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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_log_mailer_send_success() {
        let mailer = LogMailer;
        let result = mailer.send("test@example.com", "Test Subject", "Test Body").await;
        
        assert!(result.is_ok(), "LogMailer should always succeed");
    }

    #[tokio::test]
    async fn test_log_mailer_send_empty_email() {
        let mailer = LogMailer;
        let result = mailer.send("", "Subject", "Body").await;
        
        assert!(result.is_ok(), "LogMailer should handle empty email");
    }

    #[tokio::test]
    async fn test_log_mailer_send_empty_subject() {
        let mailer = LogMailer;
        let result = mailer.send("test@example.com", "", "Body").await;
        
        assert!(result.is_ok(), "LogMailer should handle empty subject");
    }

    #[tokio::test]
    async fn test_log_mailer_send_empty_body() {
        let mailer = LogMailer;
        let result = mailer.send("test@example.com", "Subject", "").await;
        
        assert!(result.is_ok(), "LogMailer should handle empty body");
    }

    #[tokio::test]
    async fn test_log_mailer_send_all_empty() {
        let mailer = LogMailer;
        let result = mailer.send("", "", "").await;
        
        assert!(result.is_ok(), "LogMailer should handle all empty fields");
    }

    #[tokio::test]
    async fn test_log_mailer_send_long_content() {
        let mailer = LogMailer;
        let long_body = "x".repeat(10000);
        let result = mailer.send("test@example.com", "Subject", &long_body).await;
        
        assert!(result.is_ok(), "LogMailer should handle long content");
    }

    #[tokio::test]
    async fn test_log_mailer_send_special_characters() {
        let mailer = LogMailer;
        let result = mailer.send(
            "test+tag@example.com",
            "Subject with 特殊文字 & symbols!",
            "Body with\nnewlines\tand\ttabs"
        ).await;
        
        assert!(result.is_ok(), "LogMailer should handle special characters");
    }

    #[tokio::test]
    async fn test_log_mailer_send_unicode() {
        let mailer = LogMailer;
        let result = mailer.send(
            "test@例え.com",
            "标题",
            "Содержимое 🎉"
        ).await;
        
        assert!(result.is_ok(), "LogMailer should handle Unicode");
    }

    #[tokio::test]
    async fn test_log_mailer_send_html_body() {
        let mailer = LogMailer;
        let html_body = r#"<html><body><h1>Test</h1><p>Paragraph</p></body></html>"#;
        let result = mailer.send("test@example.com", "HTML Test", html_body).await;
        
        assert!(result.is_ok(), "LogMailer should handle HTML content");
    }

    #[tokio::test]
    async fn test_log_mailer_multiple_sends() {
        let mailer = LogMailer;
        
        for i in 0..10 {
            let result = mailer.send(
                &format!("user{}@example.com", i),
                &format!("Subject {}", i),
                &format!("Body {}", i)
            ).await;
            
            assert!(result.is_ok(), "Each send should succeed");
        }
    }
}