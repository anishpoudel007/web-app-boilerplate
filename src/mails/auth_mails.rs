use std::sync::Arc;

use lettre::{
    transport::smtp::authentication::Credentials,
    Message, SmtpTransport, Transport as _,
};
use sailfish::TemplateSimple;

use crate::AppState;

#[derive(TemplateSimple)]
#[template(path = "user_register_email.stpl")]
struct UserRegisterTemplate {
    username: String,
}

pub fn send_register_mail(app_state: Arc<AppState>, subject: &str, to: &str) -> Result<(), String> {
    let email_body = UserRegisterTemplate {
        username: to.to_string(),
    }
    .render_once()
    .unwrap();

    let app_config = app_state.config.clone();

    let email = Message::builder()
        .from(app_config.from_email.parse().unwrap())
        .to(to.parse().unwrap())
        .subject(subject)
        .header(lettre::message::header::ContentType::TEXT_HTML)
        .body(email_body)
        .map_err(|e| e.to_string())?;

    let creds = Credentials::new(app_config.smtp_username, app_config.smtp_password);

    // Open a remote connection to gmail
    let mailer = SmtpTransport::relay(&app_config.smtp_host)
        .map_err(|e| e.to_string())?
        .credentials(creds)
        .tls(lettre::transport::smtp::client::Tls::None)
        .build();

    mailer.send(&email).map_err(|e| e.to_string())?;

    Ok(())
}
