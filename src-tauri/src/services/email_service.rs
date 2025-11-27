use super::email_renderer::{html_to_plain_text, render_email_html};
use crate::database::models::email::EmailAddress;
/// Email sending service using SMTP
use lettre::{
    message::{header::ContentType, Attachment, Mailbox, Message, MultiPart, SinglePart},
    transport::smtp::{
        authentication::Credentials,
        client::{Tls, TlsParameters},
    },
    AsyncSmtpTransport, AsyncTransport, Tokio1Executor,
};
use mime_guess::from_path;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum EmailError {
    InvalidEmail(String),
    SmtpError(String),
    BuildError(String),
    ConfigError(String),
    AttachmentError(String),
    IoError(String),
}

impl fmt::Display for EmailError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EmailError::InvalidEmail(msg) => write!(f, "Invalid email: {}", msg),
            EmailError::SmtpError(msg) => write!(f, "SMTP error: {}", msg),
            EmailError::BuildError(msg) => write!(f, "Build error: {}", msg),
            EmailError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            EmailError::AttachmentError(msg) => write!(f, "Attachment error: {}", msg),
            EmailError::IoError(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl Error for EmailError {}

impl From<lettre::error::Error> for EmailError {
    fn from(err: lettre::error::Error) -> Self {
        EmailError::SmtpError(err.to_string())
    }
}

impl From<lettre::address::AddressError> for EmailError {
    fn from(err: lettre::address::AddressError) -> Self {
        EmailError::InvalidEmail(err.to_string())
    }
}

/// SMTP configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub use_tls: bool,
}

impl SmtpConfig {
    /// Load SMTP config from environment variables
    pub fn from_env() -> Result<Self, EmailError> {
        Ok(Self {
            host: "localhost".to_string(),
            port: "587".to_string().parse().unwrap(),
            username: Option::from("".to_string()),
            password: Option::from("".to_string()),
            use_tls: true,
        })
    }
}

/// Attachment data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAttachment {
    pub filename: String,
    pub content: Vec<u8>,
    pub content_type: Option<String>,
}

/// Email data for sending
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailData {
    pub from: String,
    pub to: Vec<EmailAddress>,
    pub cc: Vec<EmailAddress>,
    pub bcc: Vec<EmailAddress>,
    pub subject: String,
    pub body_html: String,
    pub attachments: Vec<EmailAttachment>,
}

/// Email service for sending emails via SMTP
pub struct EmailService {
    config: SmtpConfig,
}

impl EmailService {
    /// Create a new email service with the given configuration
    pub fn new(config: SmtpConfig) -> Self {
        Self { config }
    }

    /// Create email service from account settings
    /// This is the recommended method for production use with multi-account support
    pub fn from_account_settings(
        smtp_host: String,
        smtp_port: u16,
        smtp_use_tls: bool,
        smtp_username: String,
        smtp_password: String,
    ) -> Result<Self, EmailError> {
        let config = SmtpConfig {
            host: smtp_host,
            port: smtp_port,
            username: Some(smtp_username),
            password: Some(smtp_password),
            use_tls: smtp_use_tls,
        };
        Ok(Self::new(config))
    }

    /// Convert EmailAddress to Mailbox
    fn to_mailbox(email_address: &EmailAddress) -> Result<Mailbox, EmailError> {
        let mailbox = if let Some(name) = &email_address.name {
            format!("{} <{}>", name, email_address.address)
                .parse()
                .map_err(|e: lettre::address::AddressError| {
                    EmailError::InvalidEmail(e.to_string())
                })?
        } else {
            email_address
                .address
                .parse()
                .map_err(|e: lettre::address::AddressError| {
                    EmailError::InvalidEmail(e.to_string())
                })?
        };
        Ok(mailbox)
    }

    /// Detect MIME type from filename using mime_guess crate
    fn detect_mime_type(filename: &str) -> ContentType {
        let mime = from_path(filename).first_or_octet_stream();

        ContentType::parse(mime.as_ref())
            .unwrap_or_else(|_| ContentType::parse("application/octet-stream").unwrap())
    }

    /// Send an email
    pub async fn send_email(&self, email_data: EmailData) -> Result<(), EmailError> {
        let from: Mailbox = email_data
            .from
            .parse()
            .map_err(|e: lettre::address::AddressError| EmailError::InvalidEmail(e.to_string()))?;

        let mut message_builder = Message::builder()
            .from(from.clone())
            .subject(email_data.subject);

        for to_addr in &email_data.to {
            message_builder = message_builder.to(Self::to_mailbox(to_addr)?);
        }

        for cc_addr in &email_data.cc {
            message_builder = message_builder.cc(Self::to_mailbox(cc_addr)?);
        }

        for bcc_addr in &email_data.bcc {
            message_builder = message_builder.bcc(Self::to_mailbox(bcc_addr)?);
        }

        let html_body = render_email_html(&email_data.body_html);
        let plain_body = html_to_plain_text(&email_data.body_html);

        let alternative_part = MultiPart::alternative()
            .singlepart(
                SinglePart::builder()
                    .header(ContentType::TEXT_PLAIN)
                    .body(plain_body),
            )
            .singlepart(
                SinglePart::builder()
                    .header(ContentType::TEXT_HTML)
                    .body(html_body),
            );

        let message = if email_data.attachments.is_empty() {
            message_builder
                .multipart(alternative_part)
                .map_err(|e| EmailError::BuildError(e.to_string()))?
        } else {
            let mut mixed = MultiPart::mixed().multipart(alternative_part);

            for attachment in &email_data.attachments {
                let content_type = if let Some(ct) = &attachment.content_type {
                    ContentType::parse(ct)
                        .unwrap_or_else(|_| Self::detect_mime_type(&attachment.filename))
                } else {
                    Self::detect_mime_type(&attachment.filename)
                };

                let attachment_part = Attachment::new(attachment.filename.clone())
                    .body(attachment.content.clone(), content_type);

                mixed = mixed.singlepart(attachment_part);
            }

            message_builder
                .multipart(mixed)
                .map_err(|e| EmailError::BuildError(e.to_string()))?
        };

        let mailer = if self.config.use_tls {
            let tls_parameters = TlsParameters::builder(self.config.host.clone())
                .build()
                .map_err(|e| EmailError::SmtpError(e.to_string()))?;

            let mut transport = AsyncSmtpTransport::<Tokio1Executor>::relay(&self.config.host)
                .map_err(|e| EmailError::SmtpError(e.to_string()))?
                .port(self.config.port);

            if let (Some(user), Some(pass)) = (&self.config.username, &self.config.password) {
                if !user.is_empty() && !pass.is_empty() {
                    transport = transport.credentials(Credentials::new(user.clone(), pass.clone()));
                }
            }

            transport.tls(Tls::Required(tls_parameters)).build()
        } else {
            let mut transport =
                AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&self.config.host)
                    .port(self.config.port);

            if let (Some(user), Some(pass)) = (&self.config.username, &self.config.password) {
                if !user.is_empty() && !pass.is_empty() {
                    transport = transport.credentials(Credentials::new(user.clone(), pass.clone()));
                }
            }

            transport.build()
        };

        mailer
            .send(message)
            .await
            .map_err(|e| EmailError::SmtpError(e.to_string()))?;

        log::info!(
            "Email sent successfully to {} recipients with {} attachment(s)",
            email_data.to.len() + email_data.cc.len() + email_data.bcc.len(),
            email_data.attachments.len()
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_mailbox() {
        let email = EmailAddress {
            address: "test@example.com".to_string(),
            name: Some("Test User".to_string()),
        };

        let mailbox = EmailService::to_mailbox(&email).unwrap();
        assert_eq!(mailbox.email.to_string(), "test@example.com");
    }

    #[test]
    fn test_to_mailbox_without_name() {
        let email = EmailAddress {
            address: "test@example.com".to_string(),
            name: None,
        };

        let mailbox = EmailService::to_mailbox(&email).unwrap();
        assert_eq!(mailbox.email.to_string(), "test@example.com");
    }
}
