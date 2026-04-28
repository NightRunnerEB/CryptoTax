use std::time::Duration;

use axum::async_trait;
use serde::Deserialize;
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

#[derive(Debug, Deserialize)]
struct TaxSvcErrorBody {
    #[serde(default)]
    details: Vec<TaxSvcErrorDetail>,
}

#[derive(Debug, Deserialize)]
struct TaxSvcErrorDetail {
    #[serde(rename = "fieldViolations", default)]
    field_violations: Vec<TaxSvcFieldViolation>,
}

#[derive(Debug, Deserialize)]
struct TaxSvcFieldViolation {
    field: String,
    description: String,
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

    fn upsert_profile_url(&self, _user_id: Uid) -> String {
        format!("{}/tax/profile", self.base_url)
    }

    fn truncate_for_log(input: &str, max_chars: usize) -> String {
        let mut out: String = input.chars().take(max_chars).collect();
        if input.chars().count() > max_chars {
            out.push_str("...");
        }
        out
    }

    fn parse_field_violation(body: &str) -> Option<(String, String)> {
        let parsed: TaxSvcErrorBody = serde_json::from_str(body).ok()?;
        for detail in parsed.details {
            for violation in detail.field_violations {
                let field = violation.field.trim();
                let description = violation.description.trim();
                if !field.is_empty() && !description.is_empty() {
                    return Some((field.to_string(), description.to_string()));
                }
            }
        }
        None
    }
}

#[async_trait]
impl TaxProfileClient for TaxSvcClient {
    async fn upsert_tax_profile(&self, user_id: Uid, profile: &RegisterTaxProfile) -> Result<(), AuthError> {
        let url = self.upsert_profile_url(user_id);

        let response =
            self.client.put(&url).header("x-user-id", user_id.to_string()).json(profile).send().await.map_err(|err| {
                warn!(user_id=%user_id, ?err, "tax-svc upsert request failed");
                AuthError::RegistrationFailed
            })?;

        if response.status().is_success() {
            return Ok(());
        }

        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        let body_for_log = Self::truncate_for_log(&body, 512);

        warn!(
            user_id = %user_id,
            status = %status.as_u16(),
            body = %body_for_log,
            "tax-svc upsert returned non-success status"
        );

        if let Some((field, description)) = Self::parse_field_violation(&body) {
            return Err(AuthError::TaxProfileFieldInvalid {
                field,
                description,
            });
        }

        Err(AuthError::RegistrationFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::TaxSvcClient;

    #[test]
    fn parse_field_violation_extracts_first_violation() {
        let body = r#"{
          "code":3,
          "message":"invalid inn",
          "details":[
            {
              "@type":"type.googleapis.com/google.rpc.BadRequest",
              "fieldViolations":[
                {"field":"inn","description":"invalid checksum","reason":"","localizedMessage":null}
              ]
            }
          ]
        }"#;

        let parsed = TaxSvcClient::parse_field_violation(body);
        assert_eq!(parsed, Some(("inn".to_string(), "invalid checksum".to_string())));
    }

    #[test]
    fn parse_field_violation_returns_none_for_unexpected_shape() {
        let body = r#"{"code":13,"message":"internal"}"#;
        assert_eq!(TaxSvcClient::parse_field_violation(body), None);
    }
}
