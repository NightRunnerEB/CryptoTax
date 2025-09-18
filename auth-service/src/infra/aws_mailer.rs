use std::time::Duration;

use async_trait::async_trait;
use aws_config::{BehaviorVersion, retry::RetryConfig, timeout::TimeoutConfig};
use aws_sdk_sesv2::{
    Client,
    types::{Body, Content, Destination, EmailContent, Message, MessageTag},
};
use tracing::{error, info, warn};

use crate::{
    auth_core::{errors::AuthError, ports::Mailer},
    config::SesConfig,
};

// SES - Amazon Simple Email Service
pub struct SesMailer {
    client: Client,
    config: SesConfig,
}

impl SesMailer {
    pub async fn new(ses_cfg: SesConfig) -> Self {
        let config = aws_config::defaults(BehaviorVersion::latest())
            .behavior_version(BehaviorVersion::latest())
            .retry_config(RetryConfig::standard().with_max_attempts(3))
            .timeout_config(
                TimeoutConfig::builder().operation_timeout(Duration::from_secs(7)).build(),
            )
            .load()
            .await;

        let client = Client::new(&config);
        Self {
            client,
            config: ses_cfg,
        }
    }

    fn build_bodies(&self, verify_link: &str) -> (String, String) {
        let text = format!(
            "Подтверждение e-mail\n\nПерейдите по ссылке: {link}\nЕсли это были не вы — проигнорируйте.",
            link = verify_link
        );
        let html = format!(
            "<h3>Подтверждение e-mail</h3>\
             <p><a href=\"{link}\">{link}</a></p>\
             <p>Если это были не вы — проигнорируйте.</p>",
            link = verify_link
        );
        (text, html)
    }

    fn mask_email(to: &str) -> String {
        // маскировка в логах: j***@domain.tld
        match to.split_once('@') {
            Some((local, dom)) if !local.is_empty() => {
                let head = &local[..local.chars().take(1).count()];
                format!("{head}***@{dom}")
            }
            _ => "***".into(),
        }
    }
}

#[async_trait]
impl Mailer for SesMailer {
    async fn send_verification(&self, to: &str, verify_link: &str) -> Result<(), AuthError> {
        let (text, html) = self.build_bodies(verify_link);

        let subject = Content::builder().data("Подтверждение e-mail").build().map_err(|e| {
            error!(?e, "failed to build subject content");
            AuthError::Internal
        })?;

        let text_part = Content::builder().data(text).build().map_err(|e| {
            error!(?e, "failed to build text body content");
            AuthError::Internal
        })?;

        let html_part = Content::builder().data(html).build().map_err(|e| {
            error!(?e, "failed to build html body content");
            AuthError::Internal
        })?;

        let msg = Message::builder()
            .subject(subject)
            .body(Body::builder().text(text_part).html(html_part).build())
            .build();

        let email_content = EmailContent::builder().simple(msg).build();
        let dest = Destination::builder().to_addresses(to).build();

        let tag =
            MessageTag::builder().name("type").value("verification").build().map_err(|e| {
                error!(?e, "failed to build message tag");
                AuthError::Internal
            })?;

        let mut req = self
            .client
            .send_email()
            .from_email_address(self.config.source.clone())
            .destination(dest)
            .content(email_content)
            .email_tags(tag);

        if let Some(cfg) = &self.config.configuration_set {
            req = req.configuration_set_name(cfg);
        }

        let res = req.send().await.map_err(|e| {
            warn!(to=%Self::mask_email(to), error=%e, "ses send_email failed");
            AuthError::EmailSendFailed
        })?;

        if let Some(id) = res.message_id() {
            info!(to=%Self::mask_email(to), %id, "ses send_email ok");
        }

        Ok(())
    }
}
