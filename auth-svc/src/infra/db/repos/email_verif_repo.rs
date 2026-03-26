use axum::async_trait;
use sqlx::{PgPool, Pool, Postgres};

use crate::auth_core::{errors::AuthError, models::Uid, ports::EmailVerificationRepo};

#[derive(Clone)]
pub struct PgEmailVerificationRepo {
    pub pool: Pool<Postgres>,
}

impl PgEmailVerificationRepo {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
        }
    }
}

#[async_trait]
impl EmailVerificationRepo for PgEmailVerificationRepo {
    async fn create_token(
        &self, user_id: Uid, token_hash: Vec<u8>, sent_to: &str, expires_at: chrono::DateTime<chrono::Utc>,
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

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};
    use serial_test::serial;

    use super::*;
    use crate::infra::db::repos::test_utils;

    #[tokio::test]
    #[ignore = "manual integration"]
    #[serial]
    async fn create_and_consume_token() {
        let pool = test_utils::test_pool().await;
        let repo = PgEmailVerificationRepo::new(pool.clone());

        let user_id = test_utils::insert_user(&pool, "Pending").await;
        let hash = test_utils::refresh_hash(3);

        repo.create_token(user_id, hash.clone(), "user@example.com", Utc::now() + Duration::hours(1))
            .await
            .expect("create token should succeed");

        let consumed = repo.consume_by_hash(&hash).await.expect("consume should work");
        assert_eq!(consumed, Some(user_id));

        let consumed_again = repo.consume_by_hash(&hash).await.expect("consume again should work");
        assert!(consumed_again.is_none(), "token should be single-use");

        test_utils::cleanup_db(&pool).await;
    }

    #[tokio::test]
    #[ignore = "manual integration"]
    #[serial]
    async fn revoke_all_for_user_invalidates_active_tokens() {
        let pool = test_utils::test_pool().await;
        let repo = PgEmailVerificationRepo::new(pool.clone());

        let user_id = test_utils::insert_user(&pool, "Pending").await;
        let hash = test_utils::refresh_hash(4);

        repo.create_token(user_id, hash.clone(), "user@example.com", Utc::now() + Duration::hours(1))
            .await
            .expect("create token should succeed");

        repo.revoke_all_for_user(user_id).await.expect("revoke should work");

        let consumed = repo.consume_by_hash(&hash).await.expect("consume query should work");
        assert!(consumed.is_none(), "revoked token must not be consumable");

        test_utils::cleanup_db(&pool).await;
    }
}
