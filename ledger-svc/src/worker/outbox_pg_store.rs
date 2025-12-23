use axum::async_trait;
use sqlx::PgPool;
use tracing::error;

use crate::domain::error::{LedgerError, Result};
use crate::infra::db::row_models::OutboxRow;
use crate::worker::outbox::{EventPublisher, OutboxStore, PublishError};

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
    async fn process_pending(&self, limit: i64, max_attempts: i32, publisher: &dyn EventPublisher) -> Result<i64> {
        let mut processed = 0_i64;
        let max_attempts = max_attempts.max(1);

        for _ in 0..limit {
            let mut tx = self.pool.begin().await.map_err(|e| LedgerError::Db(format!("outbox.begin: {e}")))?;

            let row_opt: Option<OutboxRow> = sqlx::query_as!(
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
                LIMIT 1
                "#
            )
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| LedgerError::Db(format!("outbox.select_pending_for_update: {e}")))?;

            let Some(mut row) = row_opt else {
                tx.commit().await.map_err(|e| LedgerError::Db(format!("outbox.commit.empty: {e}")))?;
                break;
            };

            row.attempts += 1;

            sqlx::query!(
                r#"
                UPDATE outbox
                   SET attempts = $2,
                       last_error = NULL
                 WHERE id = $1
                "#,
                row.id,
                row.attempts,
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| LedgerError::Db(format!("outbox.bump_attempt: {e}")))?;

            let update_result = match publisher.publish_event(&row).await {
                Ok(()) => {
                    let now = chrono::Utc::now();
                    sqlx::query!(
                        r#"
                        UPDATE outbox
                           SET status = 'published',
                               last_error = NULL,
                               published_at = $2
                         WHERE id = $1
                        "#,
                        row.id,
                        now,
                    )
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| LedgerError::Db(format!("outbox.mark_published: {e}")))
                }
                Err(PublishError::Permanent(err)) => {
                    error!("permanent publish error for outbox event_id={}: {}", row.id, err);
                    sqlx::query!(
                        r#"
                        UPDATE outbox
                           SET status = 'failed',
                               last_error = $2
                         WHERE id = $1
                        "#,
                        row.id,
                        err,
                    )
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| LedgerError::Db(format!("outbox.mark_failed_permanent: {e}")))
                }
                Err(PublishError::Transient(err)) => {
                    if row.attempts >= max_attempts {
                        let final_error = format!("max attempts reached ({max_attempts}), last transient error: {err}");
                        error!("outbox event_id={} moved to failed: {}", row.id, final_error);
                        sqlx::query!(
                            r#"
                            UPDATE outbox
                               SET status = 'failed',
                                   last_error = $2
                             WHERE id = $1
                            "#,
                            row.id,
                            final_error,
                        )
                        .execute(&mut *tx)
                        .await
                        .map_err(|e| LedgerError::Db(format!("outbox.mark_failed_max_attempts: {e}")))
                    } else {
                        error!(
                            "transient publish error for outbox event_id={}, attempts={}/{}: {}",
                            row.id, row.attempts, max_attempts, err
                        );
                        sqlx::query!(
                            r#"
                            UPDATE outbox
                               SET status = 'pending',
                                   last_error = $2
                             WHERE id = $1
                            "#,
                            row.id,
                            err,
                        )
                        .execute(&mut *tx)
                        .await
                        .map_err(|e| LedgerError::Db(format!("outbox.mark_pending_transient: {e}")))
                    }
                }
            };

            update_result?;

            tx.commit().await.map_err(|e| LedgerError::Db(format!("outbox.commit.processed: {e}")))?;

            processed += 1;
        }

        Ok(processed)
    }
}
