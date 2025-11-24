use axum::async_trait;
use sqlx::PgPool;

use crate::domain::error::{LedgerError, Result};
use crate::infra::db::row_models::OutboxRow;
use crate::worker::outbox::OutboxStore;

pub struct PgOutboxStore {
    pool: PgPool,
}

impl PgOutboxStore {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
        }
    }
}

#[async_trait]
impl OutboxStore for PgOutboxStore {
    async fn fetch_pending(&self, limit: i64) -> Result<Vec<OutboxRow>> {
        let rows: Vec<OutboxRow> = sqlx::query_as!(
            OutboxRow,
            r#"
            SELECT
                id,
                event_id,
                tenant_id,
                aggregate_type,
                aggregate_id,
                event_type,
                event_version,
                payload,
                headers,
                status,
                attempts,
                last_error,
                created_at,
                published_at
            FROM outbox
            WHERE status = 'pending'
            ORDER BY id
            FOR UPDATE SKIP LOCKED
            LIMIT $1
            "#,
            limit
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| LedgerError::Db(format!("outbox.fetch_pending: {e}")))?;

        Ok(rows)
    }

    async fn mark_published(&self, id: i32) -> Result<()> {
        let now = chrono::Utc::now();

        sqlx::query!(
            r#"
            UPDATE outbox
               SET status = 'published',
                   attempts = attempts + 1,
                   last_error = NULL,
                   published_at = $2
             WHERE id = $1
            "#,
            id,
            now,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| LedgerError::Db(format!("outbox.mark_published: {e}")))?;

        Ok(())
    }

    async fn mark_failed(&self, id: i32, error: String) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE outbox
               SET status = 'failed',
                   attempts = attempts + 1,
                   last_error = $2
             WHERE id = $1
            "#,
            id,
            error,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| LedgerError::Db(format!("outbox.mark_failed: {e}")))?;

        Ok(())
    }
}
