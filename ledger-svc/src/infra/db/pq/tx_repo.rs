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

#[cfg(test)]
mod tests {
    use serial_test::serial;
    use uuid::Uuid;

    use super::*;
    use crate::infra::db::pq::test_utils;
    use crate::infra::db::row_models::TransactionRow;

    async fn seed_transaction(pool: &sqlx::PgPool, tx: &crate::domain::models::transaction::Transaction) {
        let row = TransactionRow::from(tx);
        sqlx::query(
            r#"
            INSERT INTO transactions (
                id, tenant_id, source, time_utc, kind,
                in_money, out_money, fee_money,
                contract_symbol, derivative_kind, position_id,
                order_id, tx_hash, note, import_id, tx_fingerprint
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16)
            "#,
        )
        .bind(row.id)
        .bind(row.tenant_id)
        .bind(row.source)
        .bind(row.time_utc)
        .bind(row.kind)
        .bind(row.in_money)
        .bind(row.out_money)
        .bind(row.fee_money)
        .bind(row.contract_symbol)
        .bind(row.derivative_kind)
        .bind(row.position_id)
        .bind(row.order_id)
        .bind(row.tx_hash)
        .bind(row.note)
        .bind(row.import_id)
        .bind(row.tx_fingerprint)
        .execute(pool)
        .await
        .expect("seed transaction");
    }

    #[tokio::test]
    #[ignore = "manual integration"]
    #[serial]
    async fn list_transactions_by_import_and_tenant() {
        let pool = test_utils::test_pool().await;
        let repo = PgTransactionQueryRepository::new(pool.clone());

        let tenant_id = Uuid::new_v4();
        let import = test_utils::sample_import(tenant_id);

        sqlx::query(
            r#"
            INSERT INTO imports (id, tenant_id, source, file_name, status, total_count, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(import.id)
        .bind(import.tenant_id)
        .bind(import.source)
        .bind(import.file_name)
        .bind(import.status.to_string())
        .bind(import.total_count)
        .bind(import.created_at)
        .execute(&pool)
        .await
        .expect("seed import");

        let tx1 = test_utils::sample_transaction(tenant_id, import.id, "order-1");
        let tx2 = test_utils::sample_transaction(tenant_id, import.id, "order-2");
        seed_transaction(&pool, &tx1).await;
        seed_transaction(&pool, &tx2).await;

        let by_import = repo.list_by_import(import.id).await.expect("list_by_import should succeed");
        assert_eq!(by_import.len(), 2);

        let by_tenant_import =
            repo.list_by_tenant_import(tenant_id, import.id).await.expect("list_by_tenant_import should succeed");
        assert_eq!(by_tenant_import.len(), 2);

        let by_tenant = repo.list_for_tenant(tenant_id, 10, 0).await.expect("list_for_tenant should succeed");
        assert_eq!(by_tenant.len(), 2);
    }
}
