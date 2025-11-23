use axum::async_trait;
use sqlx::{PgPool, Pool, Postgres};

use crate::{
    auth_core::{errors::AuthError, models::*, ports::SessionRepo},
    infra::repos::PgSessionStatus,
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
