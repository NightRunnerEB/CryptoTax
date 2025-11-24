use axum::async_trait;
use chrono::Utc;
use sqlx::{PgPool, Postgres, Transaction as PgTransaction};
use uuid::Uuid;

use crate::domain::error::{LedgerError, Result};
use crate::domain::models::{import::Import, transaction::Transaction};
use crate::domain::ports::{ImportUnitOfWork, ImportUnitOfWorkFactory, OutboxRepository, TransactionCommandRepository};
use crate::infra::db::row_models::{OutboxStatus, TransactionRow};

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
                    wallet,
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
                    import_id
                )
                VALUES (
                    $1,$2,$3,$4,$5,
                    $6,$7,$8,
                    $9,$10,$11,
                    $12,$13,$14,
                    $15
                )
                "#,
                row.id,
                row.tenant_id,
                row.wallet,
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
            )
            .execute(&mut self.tx)
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
            "wallet": import.wallet,
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
        .execute(&mut self.tx)
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
        .execute(&mut self.tx)
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
