// src/lib/email.rs

use crate::{ApiError, config::EmailConfig};
use lettre::{
    message::{header::ContentType, Mailbox, Message},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Tokio1Executor,
};
use tracing::{error, info, warn};

/// Email service for sending emails
/// Supports both SMTP (production) and console logging (development)
#[derive(Clone)]
pub struct EmailService {
    config: Option<EmailConfig>,
    smtp_transport: Option<AsyncSmtpTransport<Tokio1Executor>>,
}

impl EmailService {
    /// Create new email service from configuration
    /// If email_config is None, emails will be logged to console only
    pub fn new(email_config: Option<EmailConfig>) -> Self {
        let smtp_transport = email_config.as_ref().map(|config| {
            let creds = Credentials::new(
                config.smtp_username.clone(),
                config.smtp_password.clone(),
            );

            AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp_host)
                .expect("Failed to create SMTP transport")
                .port(config.smtp_port)
                .credentials(creds)
                .build()
        });

        if email_config.is_some() {
            info!("Email service initialized with SMTP transport");
        } else {
            warn!("Email service initialized in development mode (console logging only)");
        }

        Self {
            config: email_config,
            smtp_transport,
        }
    }

    /// Check if email service is in development mode (no SMTP configured)
    pub fn is_development_mode(&self) -> bool {
        self.config.is_none()
    }

    /// Send an email using SMTP or log to console
    async fn send_email(
        &self,
        to_email: &str,
        subject: &str,
        html_body: &str,
        text_body: &str,
    ) -> Result<(), ApiError> {
        // If SMTP is configured, send actual email
        if let (Some(config), Some(transport)) = (&self.config, &self.smtp_transport) {
            let from = format!("{} <{}>", config.from_name, config.from_email)
                .parse::<Mailbox>()
                .map_err(|e| ApiError::InternalServerError(format!("Invalid from email: {}", e)))?;

            let to = to_email
                .parse::<Mailbox>()
                .map_err(|e| ApiError::BadRequest(format!("Invalid recipient email: {}", e)))?;

            let email = Message::builder()
                .from(from)
                .to(to)
                .subject(subject)
                .multipart(
                    lettre::message::MultiPart::alternative()
                        .singlepart(
                            lettre::message::SinglePart::builder()
                                .header(ContentType::TEXT_PLAIN)
                                .body(text_body.to_string()),
                        )
                        .singlepart(
                            lettre::message::SinglePart::builder()
                                .header(ContentType::TEXT_HTML)
                                .body(html_body.to_string()),
                        ),
                )
                .map_err(|e| ApiError::InternalServerError(format!("Failed to build email: {}", e)))?;

            transport
                .send(email)
                .await
                .map_err(|e| {
                    error!("Failed to send email to {}: {}", to_email, e);
                    ApiError::InternalServerError(format!("Failed to send email: {}", e))
                })?;

            info!("Email sent successfully to {}: {}", to_email, subject);
            Ok(())
        } else {
            // Development mode - log email to console
            info!(
                "=== EMAIL (Development Mode) ===\n\
                To: {}\n\
                Subject: {}\n\n\
                TEXT BODY:\n{}\n\n\
                HTML BODY:\n{}\n\
                ================================",
                to_email, subject, text_body, html_body
            );
            Ok(())
        }
    }

    /// Send password reset email
    pub async fn send_password_reset_email(
        &self,
        to_email: &str,
        username: &str,
        reset_token: &str,
        base_url: &str,
    ) -> Result<(), ApiError> {
        let reset_link = format!("{}/password-reset/{}", base_url, reset_token);
        let subject = "Reset Your Password - CrustyRustacean Dev Blog";

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
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Reset Your Password</title>
</head>
<body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333; max-width: 600px; margin: 0 auto; padding: 20px;">
    <div style="background-color: #f8f9fa; padding: 20px; border-radius: 8px;">
        <h2 style="color: #007bff; margin-top: 0;">Reset Your Password</h2>
        <p>Hi <strong>{}</strong>,</p>
        <p>You requested to reset your password for your CrustyRustacean Dev Blog account.</p>
        <p>Click the button below to reset your password:</p>
        <div style="text-align: center; margin: 30px 0;">
            <a href="{}" style="background-color: #007bff; color: white; padding: 12px 30px; text-decoration: none; border-radius: 5px; display: inline-block;">Reset Password</a>
        </div>
        <p style="color: #666; font-size: 14px;">Or copy and paste this link into your browser:<br>
        <a href="{}" style="color: #007bff;">{}</a></p>
        <p style="color: #999; font-size: 12px; margin-top: 30px; border-top: 1px solid #ddd; padding-top: 20px;">
            This link will expire in 1 hour.<br>
            If you didn't request this password reset, please ignore this email.
        </p>
        <p style="color: #666; margin-top: 20px;">Best regards,<br>
        <strong>CrustyRustacean Dev Blog Team</strong></p>
    </div>
</body>
</html>"#,
            username, reset_link, reset_link, reset_link
        );

        self.send_email(to_email, subject, &html_body, &text_body).await
    }

    /// Send welcome email
    pub async fn send_welcome_email(
        &self,
        to_email: &str,
        username: &str,
        base_url: &str,
    ) -> Result<(), ApiError> {
        let subject = "Welcome to CrustyRustacean Dev Blog!";

        let text_body = format!(
            "Hi {},\n\n\
            Welcome to CrustyRustacean Dev Blog! Your account has been created successfully.\n\n\
            You can now:\n\
            - Write and publish articles\n\
            - Follow other authors\n\
            - Comment on articles\n\
            - Save your favorites\n\n\
            Visit {} to get started!\n\n\
            Best regards,\n\
            CrustyRustacean Dev Blog Team",
            username, base_url
        );

        let html_body = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Welcome to CrustyRustacean Dev Blog</title>
</head>
<body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333; max-width: 600px; margin: 0 auto; padding: 20px;">
    <div style="background-color: #f8f9fa; padding: 20px; border-radius: 8px;">
        <h2 style="color: #007bff; margin-top: 0;">Welcome to CrustyRustacean Dev Blog!</h2>
        <p>Hi <strong>{}</strong>,</p>
        <p>Welcome to CrustyRustacean Dev Blog! Your account has been created successfully.</p>
        <div style="background-color: white; padding: 20px; border-radius: 5px; margin: 20px 0;">
            <h3 style="color: #28a745; margin-top: 0;">You can now:</h3>
            <ul style="list-style-type: none; padding-left: 0;">
                <li style="padding: 8px 0;">✓ Write and publish articles</li>
                <li style="padding: 8px 0;">✓ Follow other authors</li>
                <li style="padding: 8px 0;">✓ Comment on articles</li>
                <li style="padding: 8px 0;">✓ Save your favorites</li>
            </ul>
        </div>
        <div style="text-align: center; margin: 30px 0;">
            <a href="{}" style="background-color: #28a745; color: white; padding: 12px 30px; text-decoration: none; border-radius: 5px; display: inline-block;">Get Started</a>
        </div>
        <p style="color: #666; margin-top: 20px;">Best regards,<br>
        <strong>CrustyRustacean Dev Blog Team</strong></p>
    </div>
</body>
</html>"#,
            username, base_url
        );

        self.send_email(to_email, subject, &html_body, &text_body).await
    }

    /// Send newsletter to a single recipient
    pub async fn send_newsletter(
        &self,
        to_email: &str,
        newsletter_title: &str,
        newsletter_subject: &str,
        newsletter_body: &str,
        base_url: &str,
        unsubscribe_token: &str,
    ) -> Result<(), ApiError> {
        let unsubscribe_link = format!("{}/api/newsletters/unsubscribe/{}", base_url, unsubscribe_token);

        let text_body = format!(
            "{}\n\n\
            ---\n\
            You received this email because you subscribed to CrustyRustacean Dev Blog newsletter.\n\
            To unsubscribe, visit: {}",
            newsletter_body, unsubscribe_link
        );

        let html_body = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
</head>
<body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333; max-width: 600px; margin: 0 auto; padding: 20px;">
    <div style="background-color: #f8f9fa; padding: 20px; border-radius: 8px;">
        <div style="background-color: white; padding: 20px; border-radius: 5px;">
            {}
        </div>
        <div style="border-top: 1px solid #ddd; margin-top: 30px; padding-top: 20px; color: #666; font-size: 12px; text-align: center;">
            <p>You received this email because you subscribed to CrustyRustacean Dev Blog newsletter.</p>
            <p><a href="{}" style="color: #007bff;">Unsubscribe</a> from future newsletters.</p>
        </div>
    </div>
</body>
</html>"#,
            newsletter_title, newsletter_body, unsubscribe_link
        );

        self.send_email(to_email, newsletter_subject, &html_body, &text_body).await
    }

    /// Send newsletters to multiple recipients (bulk send)
    /// Returns number of successful sends
    pub async fn send_bulk_newsletters(
        &self,
        recipients: Vec<(String, String)>, // Vec<(email, unsubscribe_token)>
        newsletter_title: &str,
        newsletter_subject: &str,
        newsletter_body: &str,
        base_url: &str,
    ) -> Result<usize, ApiError> {
        let mut successful_sends = 0;
        let total_recipients = recipients.len();

        for (email, unsubscribe_token) in recipients {
            match self.send_newsletter(
                &email,
                newsletter_title,
                newsletter_subject,
                newsletter_body,
                base_url,
                &unsubscribe_token,
            ).await {
                Ok(_) => {
                    successful_sends += 1;
                }
                Err(e) => {
                    error!("Failed to send newsletter to {}: {}", email, e);
                    // Continue sending to other recipients even if one fails
                }
            }
        }

        info!(
            "Newsletter '{}' sent to {} out of {} recipients",
            newsletter_title,
            successful_sends,
            total_recipients
        );

        Ok(successful_sends)
    }
}
