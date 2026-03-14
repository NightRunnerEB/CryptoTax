use std::time::Duration;

use axum::async_trait;
use tracing::warn;

use crate::{
    auth_core::{
        errors::AuthError,
        models::{RegisterTaxProfile, Uid},
        ports::TaxProfileClient,
    },
    config::TaxSvcConfig,
};

pub struct TaxSvcClient {
    client: reqwest::Client,
    base_url: String,
}

impl TaxSvcClient {
    pub fn new(cfg: TaxSvcConfig) -> Result<Self, AuthError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(cfg.timeout_secs))
            .build()
            .map_err(|e| AuthError::Storage(format!("tax_svc.client_build: {e}")))?;

        Ok(Self {
            client,
            base_url: cfg.base_url.trim_end_matches('/').to_string(),
        })
    }

    fn upsert_profile_url(&self, user_id: Uid) -> String {
        format!("{}/v1/tenants/{user_id}/tax/profile", self.base_url)
    }

    fn truncate_for_log(input: &str, max_chars: usize) -> String {
        let mut out: String = input.chars().take(max_chars).collect();
        if input.chars().count() > max_chars {
            out.push_str("...");
        }
        out
    }
}

#[async_trait]
impl TaxProfileClient for TaxSvcClient {
    async fn upsert_tax_profile(&self, user_id: Uid, profile: &RegisterTaxProfile) -> Result<(), AuthError> {
        let url = self.upsert_profile_url(user_id);

        let response = self.client.put(&url).json(profile).send().await.map_err(|err| {
            warn!(user_id=%user_id, ?err, "tax-svc upsert request failed");
            AuthError::RegistrationFailed
        })?;

        if response.status().is_success() {
            return Ok(());
        }

        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        let body = Self::truncate_for_log(&body, 512);

        warn!(
            user_id = %user_id,
            status = %status.as_u16(),
            body = %body,
            "tax-svc upsert returned non-success status"
        );

        Err(AuthError::RegistrationFailed)
    }
}
