// src/lib/email.rs

//! Email sending service with trait-based abstraction.
//!
//! This module provides a flexible email sending capability that can be swapped
//! between different providers (Mailtrap for dev, SendGrid/Postmark for production).

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, warn};

// ============================================================================
// Core Types
// ============================================================================

/// Represents an email address with optional display name
#[derive(Debug, Clone, Serialize)]
pub struct EmailAddress {
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl EmailAddress {
    pub fn new(email: impl Into<String>) -> Self {
        Self {
            email: email.into(),
            name: None,
        }
    }

    pub fn with_name(email: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            email: email.into(),
            name: Some(name.into()),
        }
    }
}

/// Email body content - can be plain text, HTML, or both
#[derive(Debug, Clone)]
pub enum EmailBody {
    Text(String),
    Html(String),
    Both { text: String, html: String },
}

/// Represents an email to be sent
#[derive(Debug, Clone)]
pub struct Email {
    pub from: EmailAddress,
    pub to: Vec<EmailAddress>,
    pub subject: String,
    pub body: EmailBody,
}

impl Email {
    /// Create a new email builder
    pub fn builder() -> EmailBuilder {
        EmailBuilder::default()
    }
}

/// Builder for constructing Email instances
#[derive(Default)]
pub struct EmailBuilder {
    from: Option<EmailAddress>,
    to: Vec<EmailAddress>,
    subject: Option<String>,
    body: Option<EmailBody>,
}

impl EmailBuilder {
    pub fn from(mut self, address: EmailAddress) -> Self {
        self.from = Some(address);
        self
    }

    pub fn to(mut self, address: EmailAddress) -> Self {
        self.to.push(address);
        self
    }

    pub fn subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    pub fn text_body(mut self, text: impl Into<String>) -> Self {
        self.body = Some(EmailBody::Text(text.into()));
        self
    }

    pub fn html_body(mut self, html: impl Into<String>) -> Self {
        self.body = Some(EmailBody::Html(html.into()));
        self
    }

    pub fn body(mut self, text: impl Into<String>, html: impl Into<String>) -> Self {
        self.body = Some(EmailBody::Both {
            text: text.into(),
            html: html.into(),
        });
        self
    }

    pub fn build(self) -> Result<Email, SendError> {
        let from = self
            .from
            .ok_or_else(|| SendError::Validation("Missing 'from' address".into()))?;

        if self.to.is_empty() {
            return Err(SendError::Validation("Missing 'to' address".into()));
        }

        let subject = self
            .subject
            .ok_or_else(|| SendError::Validation("Missing subject".into()))?;

        let body = self
            .body
            .ok_or_else(|| SendError::Validation("Missing email body".into()))?;

        Ok(Email {
            from,
            to: self.to,
            subject,
            body,
        })
    }
}

/// Response from a successful email send operation
#[derive(Debug, Clone)]
pub struct SendResponse {
    /// Provider-specific message ID for tracking
    pub message_id: Option<String>,
    /// Human-readable status message
    pub message: String,
}

/// Errors that can occur during email sending
#[derive(Debug, thiserror::Error)]
pub enum SendError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("Authentication failed: {0}")]
    Authentication(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Rate limited: {0}")]
    RateLimited(String),

    #[error("Provider error: {0}")]
    Provider(String),
}

// ============================================================================
// EmailSender Trait
// ============================================================================

/// Trait for sending emails through various providers
#[async_trait]
pub trait EmailSender: Send + Sync {
    /// Send an email through the provider
    async fn send(&self, email: &Email) -> Result<SendResponse, SendError>;

    /// Check if the sender is properly configured and ready to send
    fn is_configured(&self) -> bool;
}

// ============================================================================
// Mailtrap Implementation
// ============================================================================

/// Mailtrap HTTP API email sender
///
/// Uses Mailtrap's Send API for transactional emails.
/// Supports both production and sandbox modes.
pub struct MailtrapSender {
    client: reqwest::Client,
    api_token: String,
    base_url: String,
    inbox_id: Option<String>,
}

impl MailtrapSender {
    const PRODUCTION_BASE_URL: &'static str = "https://send.api.mailtrap.io";
    const SANDBOX_BASE_URL: &'static str = "https://sandbox.api.mailtrap.io";

    /// Create a production Mailtrap sender
    pub fn new(api_token: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_token: api_token.into(),
            base_url: Self::PRODUCTION_BASE_URL.to_string(),
            inbox_id: None,
        }
    }

    /// Create a sandbox Mailtrap sender (for development/testing)
    pub fn sandbox(api_token: impl Into<String>, inbox_id: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_token: api_token.into(),
            base_url: Self::SANDBOX_BASE_URL.to_string(),
            inbox_id: Some(inbox_id.into()),
        }
    }

    /// Create with custom base URL (useful for unit testing with mocks)
    pub fn with_base_url(api_token: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_token: api_token.into(),
            base_url: base_url.into(),
            inbox_id: None,
        }
    }

    /// Get the API endpoint URL
    fn api_url(&self) -> String {
        match &self.inbox_id {
            Some(id) => format!("{}/api/send/{}", self.base_url, id),
            None => format!("{}/api/send", self.base_url),
        }
    }
}

/// Mailtrap API request payload
#[derive(Serialize)]
struct MailtrapRequest {
    from: MailtrapAddress,
    to: Vec<MailtrapAddress>,
    subject: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    html: Option<String>,
}

#[derive(Serialize)]
struct MailtrapAddress {
    email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

impl From<&EmailAddress> for MailtrapAddress {
    fn from(addr: &EmailAddress) -> Self {
        Self {
            email: addr.email.clone(),
            name: addr.name.clone(),
        }
    }
}

/// Mailtrap API success response
#[derive(Deserialize)]
struct MailtrapResponse {
    success: bool,
    message_ids: Option<Vec<String>>,
}

/// Mailtrap API error response
#[derive(Deserialize)]
struct MailtrapErrorResponse {
    errors: Option<Vec<String>>,
    error: Option<String>,
}

#[async_trait]
impl EmailSender for MailtrapSender {
    async fn send(&self, email: &Email) -> Result<SendResponse, SendError> {
        let (text, html) = match &email.body {
            EmailBody::Text(t) => (Some(t.clone()), None),
            EmailBody::Html(h) => (None, Some(h.clone())),
            EmailBody::Both { text, html } => (Some(text.clone()), Some(html.clone())),
        };

        let request = MailtrapRequest {
            from: MailtrapAddress::from(&email.from),
            to: email.to.iter().map(MailtrapAddress::from).collect(),
            subject: email.subject.clone(),
            text,
            html,
        };

        let response = self
            .client
            .post(self.api_url())
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| SendError::Network(e.to_string()))?;

        let status = response.status();

        if status.is_success() {
            let body: MailtrapResponse = response
                .json()
                .await
                .map_err(|e| SendError::Provider(format!("Failed to parse response: {}", e)))?;

            if body.success {
                Ok(SendResponse {
                    message_id: body.message_ids.and_then(|ids| ids.into_iter().next()),
                    message: "Email sent successfully".to_string(),
                })
            } else {
                Err(SendError::Provider("Send reported failure".to_string()))
            }
        } else {
            let error_body: MailtrapErrorResponse =
                response.json().await.unwrap_or(MailtrapErrorResponse {
                    errors: None,
                    error: Some("Unknown error".to_string()),
                });

            let error_msg = error_body
                .errors
                .map(|e| e.join(", "))
                .or(error_body.error)
                .unwrap_or_else(|| "Unknown error".to_string());

            match status.as_u16() {
                401 => Err(SendError::Authentication(error_msg)),
                422 => Err(SendError::Validation(error_msg)),
                429 => Err(SendError::RateLimited(error_msg)),
                _ => Err(SendError::Provider(format!(
                    "HTTP {}: {}",
                    status.as_u16(),
                    error_msg
                ))),
            }
        }
    }

    fn is_configured(&self) -> bool {
        !self.api_token.is_empty()
    }
}

// ============================================================================
// Logging Implementation (Fallback)
// ============================================================================

/// A fallback email sender that logs emails instead of sending them.
/// Useful for development when email credentials aren't configured.
pub struct LoggingEmailSender;

impl LoggingEmailSender {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LoggingEmailSender {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EmailSender for LoggingEmailSender {
    async fn send(&self, email: &Email) -> Result<SendResponse, SendError> {
        let (text, html) = match &email.body {
            EmailBody::Text(t) => (t.clone(), String::new()),
            EmailBody::Html(h) => (String::new(), h.clone()),
            EmailBody::Both { text, html } => (text.clone(), html.clone()),
        };

        let from_display = email
            .from
            .name
            .as_ref()
            .map(|n| format!("{} <{}>", n, email.from.email))
            .unwrap_or_else(|| email.from.email.clone());

        let to_display: Vec<String> = email
            .to
            .iter()
            .map(|addr| {
                addr.name
                    .as_ref()
                    .map(|n| format!("{} <{}>", n, addr.email))
                    .unwrap_or_else(|| addr.email.clone())
            })
            .collect();

        info!(
            "\n=== EMAIL (NOT SENT - LOGGING ONLY) ===\n\
            From: {}\n\
            To: {}\n\
            Subject: {}\n\
            ---\n\
            Text Body:\n{}\n\
            ---\n\
            HTML Body:\n{}\n\
            =======================================",
            from_display,
            to_display.join(", "),
            email.subject,
            text,
            html
        );

        Ok(SendResponse {
            message_id: None,
            message: "Email logged (not sent - no email provider configured)".to_string(),
        })
    }

    fn is_configured(&self) -> bool {
        false
    }
}

// ============================================================================
// Email Service Configuration
// ============================================================================

/// Configuration for the email service
#[derive(Clone, Debug)]
pub struct EmailConfig {
    pub mailtrap_api_token: Option<String>,
    /// Sandbox inbox ID - if set, uses Mailtrap sandbox instead of production
    pub mailtrap_sandbox_inbox_id: Option<String>,
    pub sender_email: String,
    pub sender_name: String,
}

impl EmailConfig {
    /// Check if Mailtrap is configured with valid credentials
    pub fn has_mailtrap(&self) -> bool {
        self.mailtrap_api_token
            .as_ref()
            .is_some_and(|t| !t.is_empty())
    }

    /// Check if sandbox mode is enabled
    pub fn is_sandbox(&self) -> bool {
        self.mailtrap_sandbox_inbox_id
            .as_ref()
            .is_some_and(|id| !id.is_empty())
    }
}

impl Default for EmailConfig {
    fn default() -> Self {
        Self {
            mailtrap_api_token: None,
            mailtrap_sandbox_inbox_id: None,
            sender_email: "noreply@example.com".to_string(),
            sender_name: "CrustyRustacean Dev Blog".to_string(),
        }
    }
}

// ============================================================================
// Email Service (High-Level API)
// ============================================================================

/// High-level email service that wraps the provider-specific sender
/// and provides convenient methods for common email types.
#[derive(Clone)]
pub struct EmailService {
    sender: Arc<dyn EmailSender>,
    pub from_email: String,
    pub from_name: String,
}

impl EmailService {
    /// Create a new email service with the given sender
    pub fn new(sender: Arc<dyn EmailSender>, from_email: String, from_name: String) -> Self {
        Self {
            sender,
            from_email,
            from_name,
        }
    }

    /// Create an email service from configuration.
    /// Returns a Mailtrap sender if configured, otherwise a logging sender.
    /// If sandbox inbox ID is set, uses Mailtrap sandbox mode.
    pub fn from_config(config: &EmailConfig) -> Self {
        let sender: Arc<dyn EmailSender> = if config.has_mailtrap() {
            let token = config.mailtrap_api_token.as_ref().unwrap();
            if config.is_sandbox() {
                let inbox_id = config.mailtrap_sandbox_inbox_id.as_ref().unwrap();
                info!(
                    "Email service configured with Mailtrap SANDBOX (inbox: {})",
                    inbox_id
                );
                Arc::new(MailtrapSender::sandbox(token, inbox_id))
            } else {
                info!("Email service configured with Mailtrap PRODUCTION");
                Arc::new(MailtrapSender::new(token))
            }
        } else {
            warn!("No email provider configured - emails will be logged only");
            Arc::new(LoggingEmailSender::new())
        };

        Self {
            sender,
            from_email: config.sender_email.clone(),
            from_name: config.sender_name.clone(),
        }
    }

    /// Check if the service has a real email provider configured
    pub fn is_configured(&self) -> bool {
        self.sender.is_configured()
    }

    /// Get the configured sender address
    fn sender_address(&self) -> EmailAddress {
        EmailAddress::with_name(&self.from_email, &self.from_name)
    }

    /// Send a password reset email
    pub async fn send_password_reset_email(
        &self,
        to_email: &str,
        username: &str,
        reset_token: &str,
        base_url: &str,
    ) -> Result<SendResponse, SendError> {
        let reset_link = format!("{}/password-reset/{}", base_url, reset_token);

        let text_body = format!(
            "Hi {},\n\n\
            You requested to reset your password for your CrustyRustacean Dev Blog account.\n\n\
            Click the link below to reset your password:\n\
            {}\n\n\
            This link will expire in 1 hour.\n\n\
            If you didn't request this password reset, please ignore this email.\n\n\
            Best regards,\n\
            CrustyRustacean Dev Blog Team",
            username, reset_link
        );

        let html_body = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Reset Your Password</title>
</head>
<body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333;">
    <div style="max-width: 600px; margin: 0 auto; padding: 20px;">
        <h2 style="color: #2c3e50;">Reset Your Password</h2>
        <p>Hi {},</p>
        <p>You requested to reset your password for your CrustyRustacean Dev Blog account.</p>
        <p>
            <a href="{}" style="display: inline-block; padding: 12px 24px; background-color: #3498db; color: white; text-decoration: none; border-radius: 4px;">
                Reset Password
            </a>
        </p>
        <p>Or copy and paste this link into your browser:</p>
        <p style="word-break: break-all; color: #7f8c8d;">{}</p>
        <p><strong>This link will expire in 1 hour.</strong></p>
        <p>If you didn't request this password reset, please ignore this email.</p>
        <hr style="border: none; border-top: 1px solid #eee; margin: 20px 0;">
        <p style="color: #7f8c8d; font-size: 12px;">
            Best regards,<br>
            CrustyRustacean Dev Blog Team
        </p>
    </div>
</body>
</html>"#,
            username, reset_link, reset_link
        );

        let email = Email::builder()
            .from(self.sender_address())
            .to(EmailAddress::new(to_email))
            .subject("Reset Your Password - CrustyRustacean Dev Blog")
            .body(text_body, html_body)
            .build()?;

        self.sender.send(&email).await
    }

    /// Send an email verification email to a newly registered user
    pub async fn send_email_verification(
        &self,
        to_email: &str,
        username: &str,
        verification_token: &str,
        base_url: &str,
    ) -> Result<SendResponse, SendError> {
        let verification_link = format!("{}/verify-email/{}", base_url, verification_token);

        let text_body = format!(
            "Hi {},\n\n\
            Thank you for registering with CrustyRustacean Dev Blog!\n\n\
            Please verify your email address by clicking the link below:\n\
            {}\n\n\
            This link will expire in 24 hours.\n\n\
            If you didn't create an account, please ignore this email.\n\n\
            Best regards,\n\
            CrustyRustacean Dev Blog Team",
            username, verification_link
        );

        let html_body = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Verify Your Email</title>
</head>
<body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333;">
    <div style="max-width: 600px; margin: 0 auto; padding: 20px;">
        <h2 style="color: #2c3e50;">Verify Your Email Address</h2>
        <p>Hi {},</p>
        <p>Thank you for registering with CrustyRustacean Dev Blog!</p>
        <p>Please verify your email address by clicking the button below:</p>
        <p>
            <a href="{}" style="display: inline-block; padding: 12px 24px; background-color: #27ae60; color: white; text-decoration: none; border-radius: 4px;">
                Verify Email
            </a>
        </p>
        <p>Or copy and paste this link into your browser:</p>
        <p style="word-break: break-all; color: #7f8c8d;">{}</p>
        <p><strong>This link will expire in 24 hours.</strong></p>
        <p>If you didn't create an account, please ignore this email.</p>
        <hr style="border: none; border-top: 1px solid #eee; margin: 20px 0;">
        <p style="color: #7f8c8d; font-size: 12px;">
            Best regards,<br>
            CrustyRustacean Dev Blog Team
        </p>
    </div>
</body>
</html>"#,
            username, verification_link, verification_link
        );

        let email = Email::builder()
            .from(self.sender_address())
            .to(EmailAddress::new(to_email))
            .subject("Verify Your Email - CrustyRustacean Dev Blog")
            .body(text_body, html_body)
            .build()?;

        self.sender.send(&email).await
    }

    /// Send a welcome email to a newly registered user
    pub async fn send_welcome_email(
        &self,
        to_email: &str,
        username: &str,
    ) -> Result<SendResponse, SendError> {
        let text_body = format!(
            "Hi {},\n\n\
            Welcome to CrustyRustacean Dev Blog! Your account has been created successfully.\n\n\
            You can now:\n\
            - Read and comment on articles\n\
            - Follow your favorite authors\n\
            - Save articles to your favorites\n\n\
            Get started by visiting our website and exploring the content.\n\n\
            Best regards,\n\
            CrustyRustacean Dev Blog Team",
            username
        );

        let html_body = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Welcome to CrustyRustacean Dev Blog</title>
</head>
<body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333;">
    <div style="max-width: 600px; margin: 0 auto; padding: 20px;">
        <h2 style="color: #2c3e50;">Welcome to CrustyRustacean Dev Blog!</h2>
        <p>Hi {},</p>
        <p>Your account has been created successfully.</p>
        <p>You can now:</p>
        <ul>
            <li>Read and comment on articles</li>
            <li>Follow your favorite authors</li>
            <li>Save articles to your favorites</li>
        </ul>
        <p>Get started by visiting our website and exploring the content.</p>
        <hr style="border: none; border-top: 1px solid #eee; margin: 20px 0;">
        <p style="color: #7f8c8d; font-size: 12px;">
            Best regards,<br>
            CrustyRustacean Dev Blog Team
        </p>
    </div>
</body>
</html>"#,
            username
        );

        let email = Email::builder()
            .from(self.sender_address())
            .to(EmailAddress::new(to_email))
            .subject("Welcome to CrustyRustacean Dev Blog!")
            .body(text_body, html_body)
            .build()?;

        self.sender.send(&email).await
    }

    /// Send a generic email
    pub async fn send(&self, email: &Email) -> Result<SendResponse, SendError> {
        self.sender.send(email).await
    }

    /// Send a newsletter subscription confirmation email
    pub async fn send_newsletter_confirmation(
        &self,
        to_email: &str,
        name: Option<&str>,
        confirmation_token: &str,
        base_url: &str,
    ) -> Result<SendResponse, SendError> {
        let confirmation_link = format!("{}/newsletter/confirmed/{}", base_url, confirmation_token);
        let greeting = name
            .map(|n| format!("Hi {},", n))
            .unwrap_or_else(|| "Hi,".to_string());

        let text_body = format!(
            "{}\n\n\
            Thank you for subscribing to the CrustyRustacean Dev Blog newsletter!\n\n\
            Please confirm your subscription by clicking the link below:\n\
            {}\n\n\
            If you didn't subscribe to our newsletter, you can safely ignore this email.\n\n\
            Best regards,\n\
            CrustyRustacean Dev Blog Team",
            greeting, confirmation_link
        );

        let html_body = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Confirm Your Newsletter Subscription</title>
</head>
<body style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; line-height: 1.6; color: #333; max-width: 600px; margin: 0 auto; padding: 20px;">
    <div style="background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); padding: 30px; border-radius: 10px 10px 0 0; text-align: center;">
        <h1 style="color: white; margin: 0; font-size: 24px;">CrustyRustacean Dev Blog</h1>
        <p style="color: rgba(255,255,255,0.9); margin: 10px 0 0 0;">Newsletter Subscription</p>
    </div>
    <div style="background: #ffffff; padding: 30px; border: 1px solid #e0e0e0; border-top: none; border-radius: 0 0 10px 10px;">
        <p style="font-size: 16px;">{}</p>
        <p>Thank you for subscribing to the CrustyRustacean Dev Blog newsletter!</p>
        <p>You'll receive:</p>
        <ul style="padding-left: 20px;">
            <li>Latest articles and tutorials</li>
            <li>Rust tips and best practices</li>
            <li>Updates from your favorite authors</li>
        </ul>
        <p>Please confirm your subscription by clicking the button below:</p>
        <div style="text-align: center; margin: 30px 0;">
            <a href="{}" style="display: inline-block; padding: 14px 28px; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; text-decoration: none; border-radius: 6px; font-weight: bold; font-size: 16px;">
                Confirm Subscription
            </a>
        </div>
        <p style="color: #666; font-size: 14px;">Or copy and paste this link into your browser:</p>
        <p style="word-break: break-all; color: #667eea; font-size: 14px;">{}</p>
        <hr style="border: none; border-top: 1px solid #eee; margin: 25px 0;">
        <p style="color: #999; font-size: 12px; text-align: center;">
            If you didn't subscribe to our newsletter, you can safely ignore this email.
        </p>
    </div>
</body>
</html>"#,
            greeting, confirmation_link, confirmation_link
        );

        let email = Email::builder()
            .from(self.sender_address())
            .to(EmailAddress::new(to_email))
            .subject("Confirm Your Newsletter Subscription - CrustyRustacean Dev Blog")
            .body(text_body, html_body)
            .build()?;

        self.sender.send(&email).await
    }

    /// Send a newsletter issue to a subscriber
    pub async fn send_newsletter_issue(
        &self,
        params: NewsletterIssueParams<'_>,
    ) -> Result<SendResponse, SendError> {
        let unsubscribe_link = format!(
            "{}/newsletter/unsubscribed/{}",
            params.base_url, params.unsubscribe_token
        );
        let greeting = params
            .subscriber_name
            .map(|n| format!("Hi {},", n))
            .unwrap_or_else(|| "Hi,".to_string());

        // Build author articles section if provided
        let author_section_text = params
            .author_articles
            .filter(|articles| !articles.is_empty())
            .map(|articles| {
                let mut section = String::from("\n\n--- FROM YOUR FAVORITE AUTHORS ---\n\n");
                for article in articles {
                    section.push_str(&format!(
                        "* {} by {}\n  {}\n  Read more: {}\n\n",
                        article.title, article.author_name, article.description, article.url
                    ));
                }
                section
            })
            .unwrap_or_default();

        let author_section_html = params.author_articles
            .filter(|articles| !articles.is_empty())
            .map(|articles| {
                let mut section = String::from(
                    r#"<div style="margin-top: 30px; padding-top: 20px; border-top: 2px solid #667eea;">
                    <h2 style="color: #667eea; font-size: 18px; margin-bottom: 20px;">From Your Favorite Authors</h2>"#,
                );
                for article in articles {
                    section.push_str(&format!(
                        r#"<div style="margin-bottom: 20px; padding: 15px; background: #f8f9fa; border-radius: 8px;">
                            <h3 style="margin: 0 0 5px 0; font-size: 16px;"><a href="{}" style="color: #333; text-decoration: none;">{}</a></h3>
                            <p style="margin: 0 0 10px 0; color: #666; font-size: 14px;">by {}</p>
                            <p style="margin: 0; color: #555; font-size: 14px;">{}</p>
                        </div>"#,
                        article.url, article.title, article.author_name, article.description
                    ));
                }
                section.push_str("</div>");
                section
            })
            .unwrap_or_default();

        let text_body = format!(
            "{}\n\n\
            {}\n\n\
            {}\
            {}\n\n\
            ---\n\
            You're receiving this because you subscribed to our newsletter.\n\
            Unsubscribe: {}\n\n\
            Best regards,\n\
            CrustyRustacean Dev Blog Team",
            greeting,
            params.newsletter_title,
            params.newsletter_body,
            author_section_text,
            unsubscribe_link
        );

        let html_body = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
</head>
<body style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; line-height: 1.6; color: #333; max-width: 600px; margin: 0 auto; padding: 20px; background: #f5f5f5;">
    <div style="background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); padding: 30px; border-radius: 10px 10px 0 0; text-align: center;">
        <h1 style="color: white; margin: 0; font-size: 24px;">CrustyRustacean Dev Blog</h1>
        <p style="color: rgba(255,255,255,0.9); margin: 10px 0 0 0;">Newsletter</p>
    </div>
    <div style="background: #ffffff; padding: 30px; border: 1px solid #e0e0e0; border-top: none;">
        <p style="font-size: 16px;">{}</p>
        <h2 style="color: #333; font-size: 22px; margin-top: 0;">{}</h2>
        <div style="font-size: 15px; line-height: 1.8;">
            {}
        </div>
        {}
    </div>
    <div style="background: #f8f9fa; padding: 20px; border: 1px solid #e0e0e0; border-top: none; border-radius: 0 0 10px 10px; text-align: center;">
        <p style="color: #666; font-size: 12px; margin: 0 0 10px 0;">
            You're receiving this email because you subscribed to our newsletter.
        </p>
        <a href="{}" style="color: #999; font-size: 12px;">Unsubscribe</a>
    </div>
</body>
</html>"#,
            params.newsletter_title,
            greeting,
            params.newsletter_title,
            params.newsletter_body.replace('\n', "<br>"),
            author_section_html,
            unsubscribe_link
        );

        let email = Email::builder()
            .from(self.sender_address())
            .to(EmailAddress::new(params.to_email))
            .subject(params.subject)
            .body(text_body, html_body)
            .build()?;

        self.sender.send(&email).await
    }
}

/// Summary of an article from a followed author for newsletter personalization
#[derive(Debug, Clone)]
pub struct AuthorArticleSummary {
    pub title: String,
    pub description: String,
    pub author_name: String,
    pub url: String,
}

/// Parameters for sending a newsletter issue email
#[derive(Debug, Clone)]
pub struct NewsletterIssueParams<'a> {
    pub to_email: &'a str,
    pub subscriber_name: Option<&'a str>,
    pub subject: &'a str,
    pub newsletter_title: &'a str,
    pub newsletter_body: &'a str,
    pub unsubscribe_token: &'a str,
    pub base_url: &'a str,
    pub author_articles: Option<&'a [AuthorArticleSummary]>,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Mock email sender for testing
    struct MockEmailSender {
        sent_emails: Mutex<Vec<Email>>,
        should_fail: bool,
    }

    impl MockEmailSender {
        fn new() -> Self {
            Self {
                sent_emails: Mutex::new(Vec::new()),
                should_fail: false,
            }
        }

        fn failing() -> Self {
            Self {
                sent_emails: Mutex::new(Vec::new()),
                should_fail: true,
            }
        }

        fn get_sent_emails(&self) -> Vec<Email> {
            self.sent_emails.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl EmailSender for MockEmailSender {
        async fn send(&self, email: &Email) -> Result<SendResponse, SendError> {
            if self.should_fail {
                return Err(SendError::Provider("Mock failure".to_string()));
            }
            self.sent_emails.lock().unwrap().push(email.clone());
            Ok(SendResponse {
                message_id: Some("mock-id-123".to_string()),
                message: "Mock email sent".to_string(),
            })
        }

        fn is_configured(&self) -> bool {
            true
        }
    }

    #[test]
    fn test_email_builder_success() {
        let email = Email::builder()
            .from(EmailAddress::with_name("sender@example.com", "Sender"))
            .to(EmailAddress::new("recipient@example.com"))
            .subject("Test Subject")
            .text_body("Hello, world!")
            .build();

        assert!(email.is_ok());
        let email = email.unwrap();
        assert_eq!(email.from.email, "sender@example.com");
        assert_eq!(email.to.len(), 1);
        assert_eq!(email.subject, "Test Subject");
    }

    #[test]
    fn test_email_builder_missing_from() {
        let result = Email::builder()
            .to(EmailAddress::new("recipient@example.com"))
            .subject("Test")
            .text_body("Body")
            .build();

        assert!(matches!(result, Err(SendError::Validation(_))));
    }

    #[test]
    fn test_email_builder_missing_to() {
        let result = Email::builder()
            .from(EmailAddress::new("sender@example.com"))
            .subject("Test")
            .text_body("Body")
            .build();

        assert!(matches!(result, Err(SendError::Validation(_))));
    }

    #[tokio::test]
    async fn test_mock_sender_success() {
        let sender = MockEmailSender::new();
        let email = Email::builder()
            .from(EmailAddress::new("test@example.com"))
            .to(EmailAddress::new("recipient@example.com"))
            .subject("Test")
            .text_body("Body")
            .build()
            .unwrap();

        let result = sender.send(&email).await;
        assert!(result.is_ok());

        let sent = sender.get_sent_emails();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].subject, "Test");
    }

    #[tokio::test]
    async fn test_mock_sender_failure() {
        let sender = MockEmailSender::failing();
        let email = Email::builder()
            .from(EmailAddress::new("test@example.com"))
            .to(EmailAddress::new("recipient@example.com"))
            .subject("Test")
            .text_body("Body")
            .build()
            .unwrap();

        let result = sender.send(&email).await;
        assert!(matches!(result, Err(SendError::Provider(_))));
    }

    #[tokio::test]
    async fn test_email_service_welcome_email() {
        let mock_sender = Arc::new(MockEmailSender::new());
        let service = EmailService::new(
            mock_sender.clone(),
            "noreply@example.com".to_string(),
            "Test App".to_string(),
        );

        let result = service
            .send_welcome_email("user@example.com", "testuser")
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_email_service_password_reset() {
        let mock_sender = Arc::new(MockEmailSender::new());
        let service = EmailService::new(
            mock_sender.clone(),
            "noreply@example.com".to_string(),
            "Test App".to_string(),
        );

        let result = service
            .send_password_reset_email(
                "user@example.com",
                "testuser",
                "reset-token-123",
                "https://example.com",
            )
            .await;

        assert!(result.is_ok());
    }

    #[test]
    fn test_email_config_has_mailtrap() {
        let config_with_token = EmailConfig {
            mailtrap_api_token: Some("valid-token".to_string()),
            ..Default::default()
        };
        assert!(config_with_token.has_mailtrap());

        let config_empty_token = EmailConfig {
            mailtrap_api_token: Some("".to_string()),
            ..Default::default()
        };
        assert!(!config_empty_token.has_mailtrap());

        let config_no_token = EmailConfig::default();
        assert!(!config_no_token.has_mailtrap());
    }

    #[test]
    fn test_logging_sender_is_not_configured() {
        let sender = LoggingEmailSender::new();
        assert!(!sender.is_configured());
    }

    #[tokio::test]
    async fn test_email_service_email_verification() {
        let mock_sender = Arc::new(MockEmailSender::new());
        let service = EmailService::new(
            mock_sender.clone(),
            "noreply@example.com".to_string(),
            "Test App".to_string(),
        );

        let result = service
            .send_email_verification(
                "user@example.com",
                "testuser",
                "verification-token-abc",
                "https://example.com",
            )
            .await;

        assert!(result.is_ok());

        // Verify the email was constructed correctly
        let sent = mock_sender.get_sent_emails();
        assert_eq!(sent.len(), 1);
        assert_eq!(
            sent[0].subject,
            "Verify Your Email - CrustyRustacean Dev Blog"
        );
        assert_eq!(sent[0].to[0].email, "user@example.com");
    }
}

// ============================================================================
// Integration Tests (using httpmock for Mailtrap API)
// ============================================================================

#[cfg(test)]
mod integration_tests {
    use super::*;
    use httpmock::prelude::*;

    fn create_test_email() -> Email {
        Email::builder()
            .from(EmailAddress::with_name("noreply@example.com", "Test App"))
            .to(EmailAddress::new("user@example.com"))
            .subject("Test Email")
            .body("Plain text body", "<p>HTML body</p>")
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn test_mailtrap_sender_success() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/api/send")
                .header("Authorization", "Bearer test-api-token")
                .header("Content-Type", "application/json");
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(serde_json::json!({
                    "success": true,
                    "message_ids": ["msg-123-abc"]
                }));
        });

        let sender = MailtrapSender::with_base_url("test-api-token", server.base_url());
        let email = create_test_email();

        let result = sender.send(&email).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.message_id, Some("msg-123-abc".to_string()));
        assert_eq!(response.message, "Email sent successfully");

        mock.assert();
    }

    #[tokio::test]
    async fn test_mailtrap_sender_authentication_failure() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(POST).path("/api/send");
            then.status(401)
                .header("Content-Type", "application/json")
                .json_body(serde_json::json!({
                    "error": "Invalid API token"
                }));
        });

        let sender = MailtrapSender::with_base_url("invalid-token", server.base_url());
        let email = create_test_email();

        let result = sender.send(&email).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, SendError::Authentication(_)));
        if let SendError::Authentication(msg) = err {
            assert!(msg.contains("Invalid API token"));
        }

        mock.assert();
    }

    #[tokio::test]
    async fn test_mailtrap_sender_validation_error() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(POST).path("/api/send");
            then.status(422)
                .header("Content-Type", "application/json")
                .json_body(serde_json::json!({
                    "errors": ["Invalid email address", "Subject too long"]
                }));
        });

        let sender = MailtrapSender::with_base_url("test-token", server.base_url());
        let email = create_test_email();

        let result = sender.send(&email).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, SendError::Validation(_)));
        if let SendError::Validation(msg) = err {
            assert!(msg.contains("Invalid email address"));
        }

        mock.assert();
    }

    #[tokio::test]
    async fn test_mailtrap_sender_rate_limited() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(POST).path("/api/send");
            then.status(429)
                .header("Content-Type", "application/json")
                .json_body(serde_json::json!({
                    "error": "Rate limit exceeded. Try again in 60 seconds."
                }));
        });

        let sender = MailtrapSender::with_base_url("test-token", server.base_url());
        let email = create_test_email();

        let result = sender.send(&email).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, SendError::RateLimited(_)));

        mock.assert();
    }

    #[tokio::test]
    async fn test_mailtrap_sender_server_error() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(POST).path("/api/send");
            then.status(500)
                .header("Content-Type", "application/json")
                .json_body(serde_json::json!({
                    "error": "Internal server error"
                }));
        });

        let sender = MailtrapSender::with_base_url("test-token", server.base_url());
        let email = create_test_email();

        let result = sender.send(&email).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, SendError::Provider(_)));
        if let SendError::Provider(msg) = err {
            assert!(msg.contains("500"));
        }

        mock.assert();
    }

    #[tokio::test]
    async fn test_mailtrap_sender_success_false_response() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(POST).path("/api/send");
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(serde_json::json!({
                    "success": false
                }));
        });

        let sender = MailtrapSender::with_base_url("test-token", server.base_url());
        let email = create_test_email();

        let result = sender.send(&email).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, SendError::Provider(_)));

        mock.assert();
    }

    #[tokio::test]
    async fn test_mailtrap_sender_request_body_format() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/api/send")
                .header("Authorization", "Bearer test-token")
                .header("Content-Type", "application/json");
            then.status(200).json_body(serde_json::json!({
                "success": true,
                "message_ids": ["msg-456"]
            }));
        });

        let sender = MailtrapSender::with_base_url("test-token", server.base_url());
        let email = create_test_email();

        let result = sender.send(&email).await;
        assert!(result.is_ok());

        // Verify the mock was called (which validates headers and path)
        mock.assert();
    }

    #[tokio::test]
    async fn test_mailtrap_sender_text_only_email() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(POST).path("/api/send");
            then.status(200).json_body(serde_json::json!({
                "success": true,
                "message_ids": ["msg-text-only"]
            }));
        });

        let sender = MailtrapSender::with_base_url("test-token", server.base_url());
        let email = Email::builder()
            .from(EmailAddress::new("sender@example.com"))
            .to(EmailAddress::new("recipient@example.com"))
            .subject("Text Only")
            .text_body("Just plain text")
            .build()
            .unwrap();

        let result = sender.send(&email).await;
        assert!(result.is_ok());

        mock.assert();
    }

    #[tokio::test]
    async fn test_mailtrap_sender_html_only_email() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(POST).path("/api/send");
            then.status(200).json_body(serde_json::json!({
                "success": true,
                "message_ids": ["msg-html-only"]
            }));
        });

        let sender = MailtrapSender::with_base_url("test-token", server.base_url());
        let email = Email::builder()
            .from(EmailAddress::new("sender@example.com"))
            .to(EmailAddress::new("recipient@example.com"))
            .subject("HTML Only")
            .html_body("<h1>HTML content</h1>")
            .build()
            .unwrap();

        let result = sender.send(&email).await;
        assert!(result.is_ok());

        mock.assert();
    }

    #[tokio::test]
    async fn test_mailtrap_sender_is_configured() {
        let sender_with_token = MailtrapSender::new("valid-token");
        assert!(sender_with_token.is_configured());

        let sender_empty_token = MailtrapSender::new("");
        assert!(!sender_empty_token.is_configured());
    }

    #[tokio::test]
    async fn test_mailtrap_sender_network_error() {
        // Use a port that's not listening to simulate network error
        let sender = MailtrapSender::with_base_url("test-token", "http://127.0.0.1:1");
        let email = create_test_email();

        let result = sender.send(&email).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, SendError::Network(_)));
    }

    #[tokio::test]
    async fn test_email_service_from_config_with_mailtrap() {
        let config = EmailConfig {
            mailtrap_api_token: Some("test-token".to_string()),
            mailtrap_sandbox_inbox_id: None,
            sender_email: "noreply@test.com".to_string(),
            sender_name: "Test Service".to_string(),
        };

        let service = EmailService::from_config(&config);

        assert!(service.is_configured());
        assert_eq!(service.from_email, "noreply@test.com");
        assert_eq!(service.from_name, "Test Service");
    }

    #[tokio::test]
    async fn test_email_service_from_config_without_mailtrap() {
        let config = EmailConfig {
            mailtrap_api_token: None,
            mailtrap_sandbox_inbox_id: None,
            sender_email: "noreply@test.com".to_string(),
            sender_name: "Test Service".to_string(),
        };

        let service = EmailService::from_config(&config);

        // Should fall back to LoggingEmailSender which is not "configured"
        assert!(!service.is_configured());
    }
}
