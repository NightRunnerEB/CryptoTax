use async_trait::async_trait;
use sqlx::{PgPool, Pool, Postgres};

use crate::auth_core::{errors::AuthError, models::*, ports::RefreshRepo};

#[derive(Clone)]
pub struct PgRefreshRepo {
    pub pool: Pool<Postgres>,
}

impl PgRefreshRepo {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
        }
    }
}

#[async_trait]
impl RefreshRepo for PgRefreshRepo {
    async fn get_by_hash(&self, hash: &[u8]) -> Result<Option<RefreshToken>, AuthError> {
        let row = sqlx::query_as!(
            RefreshRow,
            r#"
            SELECT jti, user_id, session_id, expires_at, rotated_at, revoked_at
            FROM refresh_tokens
            WHERE token_hash = $1
            "#,
            hash
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AuthError::Storage(format!("refresh.get_by_hash: {e}")))?;

        Ok(match row {
            None => None,
            Some(r) => Some(r.try_into()?),
        })
    }

    async fn mark_rotated(&self, jti: Uid) -> Result<bool, AuthError> {
        let res = sqlx::query!(
            r#"
            UPDATE refresh_tokens 
            SET rotated_at = now() 
            WHERE jti = $1
                AND rotated_at IS NULL
                AND revoked_at is NULL
                AND expires_at > now()
            "#,
            jti
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::Storage(format!("refresh.mark_rotated: {e}")))?;

        Ok(res.rows_affected() == 1)
    }

    async fn insert(&self, rec: NewRefresh) -> Result<(), AuthError> {
        sqlx::query!(
            r#"
            INSERT INTO refresh_tokens (jti, user_id, session_id, token_hash, parent_jti, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            rec.jti,
            rec.user_id,
            rec.session_id,
            rec.token_hash,
            rec.parent_jti,
            rec.expires_at
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::Storage(format!("refresh.insert: {e}")))?;

        Ok(())
    }

    async fn revoke_all_for_session(&self, session_id: Uid) -> Result<(), AuthError> {
        sqlx::query!(
            r#"
            UPDATE refresh_tokens 
            SET revoked_at = now() 
            WHERE session_id = $1 
                AND revoked_at IS NULL
            "#,
            session_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::Storage(format!("refresh.revoke_all_for_session: {e}")))?;

        Ok(())
    }
}
