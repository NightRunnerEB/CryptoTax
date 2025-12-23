use axum::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{
    error::{LedgerError, Result},
    models::transaction::Transaction,
    ports::TransactionQueryRepository,
};
use crate::infra::db::row_models::transaction::TransactionRow;

#[derive(Clone)]
pub struct PgTransactionQueryRepository {
    pool: PgPool,
}

impl PgTransactionQueryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
        }
    }
}

#[async_trait]
impl TransactionQueryRepository for PgTransactionQueryRepository {
    // этот вопрос вообще не нужен
    async fn list_by_import(&self, import_id: Uuid) -> Result<Vec<Transaction>> {
        let rows: Vec<TransactionRow> = sqlx::query_as::<_, TransactionRow>(
            r#"
            SELECT
                id, tenant_id, source, time_utc, kind,
                in_money, out_money, fee_money,
                contract_symbol, derivative_kind, position_id,
                order_id, tx_hash, note,
                import_id, tx_fingerprint
            FROM transactions
            WHERE import_id = $1
            ORDER BY time_utc ASC, id ASC
            "#,
        )
        .bind(import_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| LedgerError::Db(format!("transactions.list_by_import: {e}")))?;

        rows.into_iter()
            .map(Transaction::try_from)
            .map(|r| r.map_err(|e| LedgerError::Db(format!("TransactionRow -> Transaction: {e}"))))
            .collect()
    }

    async fn list_by_tenant_import(&self, tenant_id: Uuid, import_id: Uuid) -> Result<Vec<TransactionRow>> {
        let rows: Vec<TransactionRow> = sqlx::query_as::<_, TransactionRow>(
            r#"
            SELECT
                id, tenant_id, source, time_utc, kind,
                in_money, out_money, fee_money,
                contract_symbol, derivative_kind, position_id,
                order_id, tx_hash, note,
                import_id, tx_fingerprint
            FROM transactions
            WHERE tenant_id = $1 AND import_id = $2
            ORDER BY time_utc ASC, id ASC
            "#,
        )
        .bind(tenant_id)
        .bind(import_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| LedgerError::Db(format!("transactions.list_by_tenant_import: {e}")))?;

        return Ok(rows);
    }

    async fn list_for_tenant(&self, tenant_id: Uuid, limit: i64, offset: i64) -> Result<Vec<Transaction>> {
        let rows: Vec<TransactionRow> = sqlx::query_as::<_, TransactionRow>(
            r#"
            SELECT
                id, tenant_id, source, time_utc, kind,
                in_money, out_money, fee_money,
                contract_symbol, derivative_kind, position_id,
                order_id, tx_hash, note,
                import_id, tx_fingerprint
            FROM transactions
            WHERE tenant_id = $1
            ORDER BY time_utc ASC, id ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(tenant_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| LedgerError::Db(format!("transactions.list_for_tenant: {e}")))?;

        rows.into_iter()
            .map(Transaction::try_from)
            .map(|r| r.map_err(|e| LedgerError::Db(format!("TransactionRow -> Transaction: {e}"))))
            .collect()
    }
}
