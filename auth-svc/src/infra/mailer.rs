use async_trait::async_trait;
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    message::{Mailbox, MultiPart, SinglePart, header::ContentType},
    transport::smtp::authentication::Credentials,
};
use tokio::time::{Duration, sleep};
use tracing::{error, warn};

use crate::{
    auth_core::{errors::AuthError, ports::Mailer},
    config::SmtpConfig,
};

/// SMTP mailer (Яндекс совместим).
pub struct SmtpMailer {
    mailer: AsyncSmtpTransport<Tokio1Executor>,
    from: Mailbox,
    max_retries: u8,
}

impl SmtpMailer {
    pub fn new(cfg: SmtpConfig) -> Result<Self, AuthError> {
        let from_addr = format!("{} <{}>", cfg.display_name, cfg.username).parse::<Mailbox>().map_err(|e| {
            error!(?e, "smtp: invalid from address");
            AuthError::Internal
        })?;

        let creds = Credentials::new(cfg.username.clone(), cfg.password.clone());

        let mut builder = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&cfg.host).map_err(|e| {
            error!(?e, host=%cfg.host, "smtp: failed to build starttls transport");
            AuthError::Internal
        })?;

        if let Some(port) = cfg.port {
            builder = builder.port(port);
        }

        let mailer = builder.credentials(creds).timeout(Some(Duration::from_secs(cfg.timeout_secs))).build();

        Ok(Self {
            mailer,
            from: from_addr,
            max_retries: cfg.max_retries,
        })
    }

    fn redact_email(to: &str) -> String {
        match to.split_once('@') {
            Some((local, dom)) if local.len() > 2 => format!("{}***@{}", &local[..2], dom),
            Some((local, dom)) if !local.is_empty() => format!("*@{}", dom),
            _ => "***".into(),
        }
    }

    fn build_multipart(verify_link: &str) -> MultiPart {
        let text = format!("Подтверждение e-mail\n\nПерейдите по ссылке:\n{url}\n\nЕсли это были не вы — проигнорируйте.", url = verify_link);

        let html = format!(
            "<h3>Подтверждение e-mail</h3>\
             <p><a href=\"{link}\">{link}</a></p>\
             <p>Если это были не вы — проигнорируйте.</p>",
            link = verify_link
        );

        MultiPart::alternative()
            .singlepart(SinglePart::builder().header(ContentType::TEXT_PLAIN).body(text))
            .singlepart(SinglePart::builder().header(ContentType::TEXT_HTML).body(html))
    }
}

#[async_trait]
impl Mailer for SmtpMailer {
    async fn send_verification(&self, to: &str, verify_link: &str) -> Result<(), AuthError> {
        let to_mailbox = to.parse::<Mailbox>().map_err(|_| AuthError::Internal)?;

        let msg = Message::builder()
            .from(self.from.clone())
            .to(to_mailbox)
            .subject("Подтверждение e-mail")
            .multipart(Self::build_multipart(verify_link))
            .map_err(|e| {
                error!(?e, "smtp: failed to build message");
                AuthError::Internal
            })?;

        let mut last_err = None;
        for attempt in 1..=self.max_retries {
            match self.mailer.send(msg.clone()).await {
                Ok(_) => return Ok(()),
                Err(e) => last_err = Some(e),
            }

            if attempt < self.max_retries {
                let delay = Duration::from_millis(match attempt {
                    1 => 200,
                    2 => 500,
                    _ => 1000,
                });
                sleep(delay).await;
            }
        }

        if let Some(e) = last_err {
            warn!(to=%Self::redact_email(to), error=%e, "smtp: send failed after retries");
        }

        Err(AuthError::EmailSendFailed)
    }
}
