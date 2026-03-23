use axum::async_trait;
use chrono::Utc;
use sqlx::{PgPool, Postgres, Transaction as PgTransaction};
use uuid::Uuid;

use crate::domain::error::{LedgerError, Result};
use crate::domain::models::{import::Import, transaction::Transaction};
use crate::domain::ports::{ImportUnitOfWork, ImportUnitOfWorkFactory, OutboxRepository, TransactionCommandRepository};
use crate::infra::db::row_models::{OutboxStatus, TransactionRow};

#[derive(Clone)]
pub struct PgImportUnitOfWorkFactory {
    pool: PgPool,
}

impl PgImportUnitOfWorkFactory {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
        }
    }
}

#[async_trait]
impl ImportUnitOfWorkFactory for PgImportUnitOfWorkFactory {
    async fn begin(&self) -> Result<Box<dyn ImportUnitOfWork>> {
        let tx = self.pool.begin().await.map_err(|e| LedgerError::Db(format!("begin import UoW: {e}")))?;

        Ok(Box::new(PgImportUnitOfWork {
            tx,
        }))
    }
}

pub struct PgImportUnitOfWork<'a> {
    tx: PgTransaction<'a, Postgres>,
}

#[async_trait]
impl<'a> TransactionCommandRepository for PgImportUnitOfWork<'a> {
    async fn insert_batch(&mut self, txs: &[Transaction]) -> Result<()> {
        if txs.is_empty() {
            return Ok(());
        }

        for t in txs {
            let row = TransactionRow::from(t);

            sqlx::query!(
                r#"
                INSERT INTO transactions (
                    id,
                    tenant_id,
                    source,
                    time_utc,
                    kind,
                    in_money,
                    out_money,
                    fee_money,
                    contract_symbol,
                    derivative_kind,
                    position_id,
                    order_id,
                    tx_hash,
                    note,
                    import_id,
                    tx_fingerprint
                )
                VALUES (
                    $1,$2,$3,$4,$5,
                    $6,$7,$8,
                    $9,$10,$11,
                    $12,$13,$14,
                    $15, $16
                )
                ON CONFLICT (tx_fingerprint) DO NOTHING
                "#,
                row.id,
                row.tenant_id,
                row.source,
                row.time_utc,
                row.kind,
                row.in_money,
                row.out_money,
                row.fee_money,
                row.contract_symbol,
                row.derivative_kind,
                row.position_id,
                row.order_id,
                row.tx_hash,
                row.note,
                row.import_id,
                row.tx_fingerprint
            )
            .execute(&mut *self.tx)
            .await
            .map_err(|e| LedgerError::Db(format!("transactions.insert_batch: {e}")))?;
        }

        Ok(())
    }
}

#[async_trait]
impl<'a> OutboxRepository for PgImportUnitOfWork<'a> {
    async fn enqueue_transactions_imported(&mut self, import: &Import) -> Result<()> {
        let event_id = Uuid::new_v4();
        let now = Utc::now();
        let status = OutboxStatus::Pending.to_string();

        let payload = serde_json::json!({
            "tenant_id": import.tenant_id,
            "import_id": import.id,
            "source": import.source,
            "total_count": import.total_count,
            "created_at": import.created_at,
        });

        sqlx::query!(
            r#"
            INSERT INTO outbox (
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
            )
            VALUES (
                $1,$2,
                $3,$4,
                $5,$6,
                $7,$8,
                $9,$10,$11,
                $12,$13
            )
            "#,
            event_id,
            import.tenant_id,
            "import",                // aggregate_type
            import.id,               // aggregate_id
            "transactions.imported", // event_type
            1,                       // event_version
            payload,
            Option::<serde_json::Value>::None, // headers
            status,
            0,                      // attempts
            Option::<String>::None, // last_error
            now,
            Option::<chrono::DateTime<Utc>>::None, // published_at
        )
        .execute(&mut *self.tx)
        .await
        .map_err(|e| LedgerError::Db(format!("outbox.enqueue_transactions_imported: {e}")))?;

        Ok(())
    }
}

#[async_trait]
impl<'a> ImportUnitOfWork for PgImportUnitOfWork<'a> {
    fn transactions(&mut self) -> &mut dyn TransactionCommandRepository {
        self
    }

    fn outbox(&mut self) -> &mut dyn OutboxRepository {
        self
    }

    async fn mark_import_completed(&mut self, import_id: Uuid, total_count: i32) -> Result<()> {
        let now = Utc::now();

        sqlx::query!(
            r#"
            UPDATE imports
               SET status = 'completed',
                   total_count = $2,
                   completed_at = COALESCE(completed_at, $3)
             WHERE id = $1
            "#,
            import_id,
            total_count,
            now,
        )
        .execute(&mut *self.tx)
        .await
        .map_err(|e| LedgerError::Db(format!("imports.mark_completed: {e}")))?;

        Ok(())
    }

    async fn commit(mut self: Box<Self>) -> Result<()> {
        self.tx.commit().await.map_err(|e| LedgerError::Db(format!("uow.commit: {e}")))?;
        Ok(())
    }

    async fn rollback(mut self: Box<Self>) -> Result<()> {
        self.tx.rollback().await.map_err(|e| LedgerError::Db(format!("uow.rollback: {e}")))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use serial_test::serial;
    use uuid::Uuid;

    use super::*;
    use crate::domain::ports::{ImportCommandRepository, ImportUnitOfWorkFactory};
    use crate::infra::db::pq::{import_repo::PgImportRepository, test_utils};

    #[tokio::test]
    #[ignore = "manual integration"]
    #[serial]
    async fn uow_commit_persists_transactions_import_status_and_outbox() {
        let pool = test_utils::test_pool().await;
        let import_repo = PgImportRepository::new(pool.clone());
        let uow_factory = PgImportUnitOfWorkFactory::new(pool.clone());

        let tenant_id = Uuid::new_v4();
        let mut import = test_utils::sample_import(tenant_id);
        import_repo.insert_processing(&import).await.expect("insert processing import");

        let tx = test_utils::sample_transaction(tenant_id, import.id, "order-uow-1");
        import.total_count = 1;

        let mut uow = uow_factory.begin().await.expect("begin uow");
        uow.transactions().insert_batch(&[tx.clone()]).await.expect("insert tx batch");
        uow.outbox().enqueue_transactions_imported(&import).await.expect("enqueue outbox");
        uow.mark_import_completed(import.id, 1).await.expect("mark import completed");
        uow.commit().await.expect("commit uow");

        let tx_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM transactions WHERE import_id = $1")
            .bind(import.id)
            .fetch_one(&pool)
            .await
            .expect("count transactions");
        assert_eq!(tx_count, 1);

        let outbox_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM outbox WHERE aggregate_id = $1")
            .bind(import.id)
            .fetch_one(&pool)
            .await
            .expect("count outbox");
        assert_eq!(outbox_count, 1);

        let status: String = sqlx::query_scalar("SELECT status FROM imports WHERE id = $1")
            .bind(import.id)
            .fetch_one(&pool)
            .await
            .expect("fetch import status");
        assert_eq!(status, "completed");
    }

    #[tokio::test]
    #[ignore = "manual integration"]
    #[serial]
    async fn uow_rollback_discards_inserted_transactions() {
        let pool = test_utils::test_pool().await;
        let import_repo = PgImportRepository::new(pool.clone());
        let uow_factory = PgImportUnitOfWorkFactory::new(pool.clone());

        let tenant_id = Uuid::new_v4();
        let import = test_utils::sample_import(tenant_id);
        import_repo.insert_processing(&import).await.expect("insert processing import");

        let tx = test_utils::sample_transaction(tenant_id, import.id, "order-uow-rollback");

        let mut uow = uow_factory.begin().await.expect("begin uow");
        uow.transactions().insert_batch(&[tx.clone()]).await.expect("insert tx batch");
        uow.rollback().await.expect("rollback uow");

        let tx_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM transactions WHERE import_id = $1")
            .bind(import.id)
            .fetch_one(&pool)
            .await
            .expect("count transactions");
        assert_eq!(tx_count, 0);
    }
}
