use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
             message::header::ContentType,
            transport::smtp::authentication::Credentials,};

use std::env;

pub async fn send_rst_email(
    recipient: &str,
    reset_url: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync >> {
    let host = env::var("SMTP_HOST")?;
    let port: u16 = env::var("SMTP_PORT")?.parse()?;
    let username = env::var("SMTP_USERNAME")?;
    let password = env::var("SMTP_PASSWORD")?;
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

    let credentials = Credentials::new(username, password);

    let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&host)?
    .port(port)
    .credentials(credentials)
    .build();

    mailer.send(email).await?;

    Ok(())
}