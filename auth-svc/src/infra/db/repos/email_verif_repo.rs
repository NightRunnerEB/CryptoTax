use async_trait::async_trait;
use sqlx::{PgPool, Pool, Postgres};

use crate::auth_core::{errors::AuthError, models::Uid, ports::EmailVerificationRepo};

#[derive(Clone)]
pub struct PgEmailVerificationRepo {
    pub pool: Pool<Postgres>,
}

impl PgEmailVerificationRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EmailVerificationRepo for PgEmailVerificationRepo {
    async fn create_token(
        &self,
        user_id: Uid,
        token_hash: Vec<u8>,
        sent_to: &str,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), AuthError> {
        sqlx::query!(
            r#"
            INSERT INTO email_verifications (user_id, token_hash, sent_to, expires_at)
            VALUES ($1, $2, $3, $4)
            "#,
            user_id,
            token_hash, // bytea
            sent_to,    // text
            expires_at
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::Storage(format!("email_verifications.insert: {e}")))?;

        Ok(())
    }

    async fn revoke_all_for_user(&self, user_id: Uid) -> Result<(), AuthError> {
        sqlx::query!(
            r#"
            UPDATE email_verifications
            SET consumed_at = now()
            WHERE user_id = $1
              AND consumed_at IS NULL
              AND expires_at > now()
            "#,
            user_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::Storage(format!("email_verifications.revoke_all_for_user: {e}")))?;

        Ok(())
    }

    async fn consume_by_hash(&self, token_hash: &[u8]) -> Result<Option<Uid>, AuthError> {
        let rec = sqlx::query!(
            r#"
            UPDATE email_verifications
            SET consumed_at = now()
            WHERE token_hash = $1
              AND consumed_at IS NULL
              AND expires_at > now()
            RETURNING user_id
            "#,
            token_hash
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AuthError::Storage(format!("email_verifications.consume_by_hash: {e}")))?;

        Ok(rec.map(|r| r.user_id))
    }
}
