use axum::async_trait;
use sqlx::{PgPool, Pool, Postgres};

use crate::{
    auth_core::{errors::AuthError, models::*, ports::RefreshRepo},
    infra::repos::RefreshRow,
};

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

#[cfg(test)]
mod tests {
    use serial_test::serial;
    use uuid::Uuid;

    use super::*;
    use crate::auth_core::models::NewRefresh;
    use crate::infra::db::repos::test_utils;

    #[tokio::test]
    #[ignore = "manual integration"]
    #[serial]
    async fn insert_get_mark_rotated_and_revoke() {
        let pool = test_utils::test_pool().await;
        let repo = PgRefreshRepo::new(pool.clone());

        let user_id = test_utils::insert_user(&pool, "Active").await;
        let session_id = test_utils::insert_session(&pool, user_id).await;

        let first = NewRefresh {
            jti: Uuid::new_v4(),
            user_id,
            session_id,
            token_hash: test_utils::refresh_hash(1),
            parent_jti: None,
            expires_at: test_utils::refresh_expiry(),
        };

        repo.insert(first.clone()).await.expect("insert first refresh");

        let found = repo.get_by_hash(&first.token_hash).await.expect("get_by_hash should work").expect("token should exist");
        assert_eq!(found.jti, first.jti);
        assert!(found.rotated_at.is_none());
        assert!(found.revoked_at.is_none());

        let rotated = repo.mark_rotated(first.jti).await.expect("mark_rotated should work");
        assert!(rotated);
        let rotated_again = repo.mark_rotated(first.jti).await.expect("second mark_rotated should work");
        assert!(!rotated_again);

        let second = NewRefresh {
            jti: Uuid::new_v4(),
            user_id,
            session_id,
            token_hash: test_utils::refresh_hash(2),
            parent_jti: Some(first.jti),
            expires_at: test_utils::refresh_expiry(),
        };
        repo.insert(second.clone()).await.expect("insert second refresh");

        repo.revoke_all_for_session(session_id).await.expect("revoke session tokens");
        let revoked = repo.get_by_hash(&second.token_hash).await.expect("get_by_hash after revoke").expect("token should exist");
        assert!(revoked.revoked_at.is_some(), "token should be revoked");

        test_utils::cleanup_db(&pool).await;
    }

    #[tokio::test]
    #[ignore = "manual integration"]
    #[serial]
    async fn get_by_hash_returns_none_for_unknown_token() {
        let pool = test_utils::test_pool().await;
        let repo = PgRefreshRepo::new(pool.clone());

        let missing = repo.get_by_hash(&test_utils::refresh_hash(7)).await.expect("query should succeed");
        assert!(missing.is_none());

        test_utils::cleanup_db(&pool).await;
    }
}
