// src/lib/email.rs

use crate::ApiError;
use tracing::info;

/// Email service for sending emails
/// Currently logs emails to console
/// TODO: Integrate with SMTP or email service provider
#[derive(Clone, Debug)]
pub struct EmailService {
    pub from_email: String,
    pub from_name: String,
}

impl EmailService {
    pub fn new(from_email: String, from_name: String) -> Self {
        Self {
            from_email,
            from_name,
        }
    }

    /// Send password reset email
    /// Currently logs to console
    /// TODO: Integrate actual email sending
    pub async fn send_password_reset_email(
        &self,
        to_email: &str,
        username: &str,
        reset_token: &str,
        base_url: &str,
    ) -> Result<(), ApiError> {
        let reset_link = format!("{}/password-reset/{}", base_url, reset_token);

        // For now, just log the email content
        // In production, this would send an actual email
        info!(
            "=== PASSWORD RESET EMAIL ===\n\
            To: {}\n\
            From: {} <{}>\n\
            Subject: Reset Your Password - CrustyRustacean Dev Blog\n\n\
            Hi {},\n\n\
            You requested to reset your password for your CrustyRustacean Dev Blog account.\n\n\
            Click the link below to reset your password:\n\
            {}\n\n\
            This link will expire in 1 hour.\n\n\
            If you didn't request this password reset, please ignore this email.\n\n\
            Best regards,\n\
            CrustyRustacean Dev Blog Team\n\
            ===========================",
            to_email, self.from_name, self.from_email, username, reset_link
        );

        Ok(())
    }

    /// Send welcome email
    /// Currently logs to console
    /// TODO: Integrate actual email sending
    pub async fn send_welcome_email(&self, to_email: &str, username: &str) -> Result<(), ApiError> {
        info!(
            "=== WELCOME EMAIL ===\n\
            To: {}\n\
            From: {} <{}>\n\
            Subject: Welcome to CrustyRustacean Dev Blog!\n\n\
            Hi {},\n\n\
            Welcome to CrustyRustacean Dev Blog! Your account has been created successfully.\n\n\
            You can now:\n\
            - Write and publish articles\n\
            - Follow other authors\n\
            - Comment on articles\n\
            - Save your favorites\n\n\
            Get started by visiting our website and logging in with your credentials.\n\n\
            Best regards,\n\
            CrustyRustacean Dev Blog Team\n\
            =====================",
            to_email, self.from_name, self.from_email, username
        );

        Ok(())
    }
}
