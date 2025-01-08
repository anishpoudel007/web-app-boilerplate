use lettre::{
    message::{header::ContentType, MultiPart},
    transport::smtp::authentication::Credentials,
    Message, SmtpTransport, Transport as _,
};
use sailfish::TemplateSimple;

#[derive(TemplateSimple)]
#[template(path = "user_register_email.stpl")]
struct UserRegisterTemplate {
    username: String,
}

pub fn send_register_mail(subject: &str, to: &str) -> Result<(), String> {
    let ctx = UserRegisterTemplate {
        username: to.to_string(),
    };

    let email_body = ctx.render_once().unwrap();

    let from_email = std::env::var("FROM_EMAIL").unwrap();

    let email = Message::builder()
        .from(from_email.parse().unwrap())
        .to(to.parse().unwrap())
        .subject(subject)
        .header(lettre::message::header::ContentType::TEXT_HTML)
        .body(email_body)
        .map_err(|e| e.to_string())?;

    let username = std::env::var("SMTP_USERNAME").unwrap();
    let password = std::env::var("SMTP_PASSWORD").unwrap();
    let host = std::env::var("SMTP_HOST").unwrap();

    let creds = Credentials::new(username, password);

    // Open a remote connection to gmail
    let mailer = SmtpTransport::relay(&host)
        .map_err(|e| e.to_string())?
        .credentials(creds)
        .tls(lettre::transport::smtp::client::Tls::None)
        .build();

    mailer.send(&email).map_err(|e| e.to_string())?;

    Ok(())
}
