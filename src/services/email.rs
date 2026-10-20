use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor, message::header::ContentType,
    transport::smtp::authentication::Credentials,
};

use std::env;

pub async fn send_rst_email(
    recipient: &str,
    reset_url: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let host = env::var("SMTP_HOST")?;
    let port: u16 = env::var("SMTP_PORT")?.parse()?;
    let security = env::var("SMTP_SECURITY").unwrap_or_else(|_| "starttls".to_owned());
    let username = env::var("SMTP_USERNAME").unwrap_or_default();
    let password = env::var("SMTP_PASSWORD").unwrap_or_default();
    let from = env::var("SMTP_FROM_EMAIL")?;

    let email = Message::builder()
        .from(from.parse()?)
        .to(recipient.parse()?)
        .subject("Reset your CloudLedger Password")
        .header(ContentType::TEXT_PLAIN)
        .body(format!(
            "A CloudLedger Password Reset was Requested.\n\n\
        Use this link within 15 minutes: \n{reset_url}\n\n
        If you did not request this, ignore this email."
        ))?;

    let mailer = match security.as_str() {
        "none" => {
            if !matches!(host.as_str(), "127.0.0.1" | "localhost" | "::1") {
                return Err("SMTP_SECURITY=none is allowed only for localhost".into());
            }

            if !username.is_empty() || !password.is_empty() {
                return Err("Do not configure SMTP credentials with security=none".into());
            }

            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&host)
                .port(port)
                .build()
        }

        "starttls" => {
            if username.is_empty() || password.is_empty() {
                return Err("SMTP username and password are required for STARTTLS".into());
            }

            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&host)?
                .port(port)
                .credentials(Credentials::new(username, password))
                .build()
        }

        "tls" => {
            if username.is_empty() || password.is_empty() {
                return Err("SMTP username and password are required for TLS".into());
            }

            AsyncSmtpTransport::<Tokio1Executor>::relay(&host)?
                .port(port)
                .credentials(Credentials::new(username, password))
                .build()
        }

        _ => return Err("Invalid SMTP_SECURITY; use none, starttls, or tls".into()),
    };

    mailer.send(email).await?;

    Ok(())
}
