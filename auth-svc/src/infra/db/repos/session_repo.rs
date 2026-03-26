use axum::async_trait;
use sqlx::{PgPool, Pool, Postgres};

use crate::{
    auth_core::{errors::AuthError, models::*, ports::SessionRepo},
    infra::repos::{PgSessionStatus, SessionRow},
};

#[derive(Clone)]
pub struct PgSessionRepo {
    pub pool: Pool<Postgres>,
}

impl PgSessionRepo {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
        }
    }
}

#[async_trait]
impl SessionRepo for PgSessionRepo {
    async fn create(&self, user_id: Uid, ip: Option<String>, ua: Option<String>) -> Result<Session, AuthError> {
        let r = sqlx::query_as!(
            SessionRow,
            r#"
            INSERT INTO sessions (user_id, ip, user_agent)
            VALUES ($1, $2, $3)
            RETURNING id, user_id, status::text AS "status!", created_at, last_seen_at, ip, user_agent
            "#,
            user_id,
            ip,
            ua
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AuthError::Storage(e.to_string()))?;

        Ok(r.try_into()?)
    }

    async fn get(&self, session_id: Uid) -> Result<Option<Session>, AuthError> {
        let row = sqlx::query_as!(
            SessionRow,
            r#"
            SELECT id, user_id, status::text AS "status!", created_at, last_seen_at, ip, user_agent
            FROM sessions WHERE id = $1
            "#,
            session_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AuthError::Storage(format!("sessions.get: {e}")))?;

        Ok(match row {
            None => None,
            Some(r) => Some(r.try_into()?),
        })
    }

    async fn set_status(&self, session_id: Uid, status: SessionStatus) -> Result<(), AuthError> {
        let pg_status: PgSessionStatus = status.into();
        let _ = sqlx::query!(
            r#"
            UPDATE sessions
            SET status = $2::session_status
            WHERE id = $1
            "#,
            session_id,
            pg_status as PgSessionStatus
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::Storage(format!("sessions.set_status: {e}")))?;
        Ok(())
    }

    async fn touch(&self, session_id: Uid) -> Result<(), AuthError> {
        let _ = sqlx::query!(
            r#"
            UPDATE sessions
            SET last_seen_at = now()
            WHERE id = $1
            "#,
            session_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::Storage(format!("sessions.touch: {e}")))?;
        Ok(())
    }

    async fn list_for_user(&self, user_id: Uid) -> Result<Vec<Session>, AuthError> {
        let rows = sqlx::query_as!(
            SessionRow,
            r#"
            SELECT id, user_id, status::text AS "status!", created_at, last_seen_at, ip, user_agent
            FROM sessions WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AuthError::Storage(format!("sessions.list_for_user: {e}")))?;

        let mut out = Vec::with_capacity(rows.len());
        for r in rows {
            out.push(r.try_into()?);
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use serial_test::serial;

    use super::*;
    use crate::auth_core::models::SessionStatus;
    use crate::infra::db::repos::test_utils;

    #[tokio::test]
    #[ignore = "manual integration"]
    #[serial]
    async fn create_get_list_set_status_touch() {
        let pool = test_utils::test_pool().await;
        let repo = PgSessionRepo::new(pool.clone());

        let user_id = test_utils::insert_user(&pool, "Active").await;
        let created = repo
            .create(user_id, Some("127.0.0.1".to_string()), Some("tests".to_string()))
            .await
            .expect("session create should work");
        assert!(matches!(created.status, SessionStatus::Active));
        assert_eq!(created.user_id, user_id);

        let fetched = repo.get(created.id).await.expect("get should work").expect("session exists");
        assert_eq!(fetched.id, created.id);
        assert!(matches!(fetched.status, SessionStatus::Active));

        repo.touch(created.id).await.expect("touch should work");

        repo.set_status(created.id, SessionStatus::Revoked).await.expect("set_status should work");
        let revoked = repo.get(created.id).await.expect("get after revoke").expect("session exists");
        assert!(matches!(revoked.status, SessionStatus::Revoked));

        let list = repo.list_for_user(user_id).await.expect("list_for_user should work");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, created.id);

        test_utils::cleanup_db(&pool).await;
    }
}
