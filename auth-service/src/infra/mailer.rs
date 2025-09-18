use async_trait::async_trait;
use lettre::{Message, SmtpTransport, Transport, transport::smtp::authentication::Credentials};
use tracing::{error, info, warn};

use crate::{
    auth_core::{errors::AuthError, ports::Mailer},
    config::SmtpConfig,
};

/// SMTP mailer (Яндекс)
pub struct SmtpMailer {
    mailer: SmtpTransport,
    config: SmtpConfig,
}

impl SmtpMailer {
    pub fn new(cfg: SmtpConfig) -> Self {
        let creds = Credentials::new(cfg.username.clone(), cfg.password.clone());

        let mailer = SmtpTransport::relay("smtp.yandex.ru")
            .expect("smtp.yandex.ru unreachable")
            .credentials(creds)
            .build();

        Self {
            mailer,
            config: cfg,
        }
    }

    fn build_body(verify_link: &str) -> String {
        let html = format!(
            "<h3>Подтверждение e-mail</h3>\
             <p><a href=\"{link}\">{link}</a></p>\
             <p>Если это были не вы — проигнорируйте.</p>",
            link = verify_link
        );

        html
    }

    fn mask_email(to: &str) -> String {
        match to.split_once('@') {
            Some((local, dom)) if !local.is_empty() => {
                let head = local.chars().next().unwrap_or('*');
                format!("{head}***@{dom}")
            }
            _ => "***".into(),
        }
    }
}

#[async_trait]
impl Mailer for SmtpMailer {
    async fn send_verification(&self, to: &str, verify_link: &str) -> Result<(), AuthError> {
        let html = Self::build_body(verify_link);

        let email = Message::builder()
            .from(
                format!("{} <{}>", self.config.display_name, self.config.username)
                    .parse()
                    .map_err(|e| {
                        error!(?e, "invalid from address");
                        AuthError::Internal
                    })?,
            )
            .to(to.parse().map_err(|e| {
                error!(?e, "invalid to address");
                AuthError::Internal
            })?)
            .subject("Подтверждение e-mail")
            .header(lettre::message::header::ContentType::TEXT_HTML)
            .body(html)
            .map_err(|e| {
                error!(?e, "failed to build email message");
                AuthError::Internal
            })?;

        match self.mailer.send(&email) {
            Ok(res) => {
                info!(to=%Self::mask_email(to), response=?res, "SMTP send ok");
                Ok(())
            }
            Err(e) => {
                warn!(to=%Self::mask_email(to), error=%e, "SMTP send failed");
                Err(AuthError::EmailSendFailed)
            }
        }
    }
}
