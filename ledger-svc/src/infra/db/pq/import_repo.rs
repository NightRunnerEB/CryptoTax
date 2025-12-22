use axum::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{
    error::{LedgerError, Result},
    models::import::Import,
    ports::{ImportCommandRepository, ImportQueryRepository},
};
use crate::infra::db::row_models::ImportRow;

#[derive(Clone)]
pub struct PgImportRepository {
    pool: PgPool,
}

impl PgImportRepository {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
        }
    }
}

#[async_trait]
impl ImportCommandRepository for PgImportRepository {
    async fn insert_processing(&self, import: &Import) -> Result<()> {
        let row = ImportRow::from(import);

        sqlx::query!(
            r#"
            INSERT INTO imports (
                id,
                tenant_id,
                source,
                file_name,
                status,
                total_count,
                error_summary,
                error_details,
                created_at,
                completed_at
            )
            VALUES (
                $1,$2,$3,$4,$5,
                $6,$7,$8,$9,$10
            )
            "#,
            row.id,
            row.tenant_id,
            row.source,
            row.file_name,
            row.status,
            row.total_count,
            row.error_summary,
            row.error_details,
            row.created_at,
            row.completed_at,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| LedgerError::Db(format!("imports.insert_pending: {e}")))?;

        Ok(())
    }

    async fn mark_failed(&self, import_id: Uuid, error_summary: String) -> Result<()> {
        let now = chrono::Utc::now();

        sqlx::query!(
            r#"
            UPDATE imports
               SET status = 'failed',
                   error_summary = $2,
                   completed_at = COALESCE(completed_at, $3)
             WHERE id = $1
            "#,
            import_id,
            error_summary,
            now,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| LedgerError::Db(format!("imports.mark_failed: {e}")))?;

        Ok(())
    }
}

#[async_trait]
impl ImportQueryRepository for PgImportRepository {
    async fn get(&self, id: Uuid) -> Result<Option<Import>> {
        let row: Option<ImportRow> = sqlx::query_as::<_, ImportRow>(
            r#"
            SELECT
                id,
                tenant_id,
                source,
                file_name,
                status,
                total_count,
                error_summary,
                error_details,
                created_at,
                completed_at
            FROM imports
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| LedgerError::Db(format!("imports.get: {e}")))?;

        let Some(row) = row else {
            return Ok(None);
        };

        Import::try_from(row).map(Some).map_err(|e| LedgerError::Db(format!("ImportRow -> Import conversion failed: {e}")))
    }

    async fn list_for_tenant(self: &Self, tenant_id: Uuid, limit: i64, offset: i64) -> Result<Vec<Import>> {
        let rows: Vec<ImportRow> = sqlx::query_as::<_, ImportRow>(
            r#"
            SELECT
                id,
                tenant_id,
                source,
                file_name,
                status,
                total_count,
                error_summary,
                error_details,
                created_at,
                completed_at
            FROM imports
            WHERE tenant_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(tenant_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| LedgerError::Db(format!("imports.list_for_tenant: {e}")))?;

        rows.into_iter()
            .map(Import::try_from)
            .map(|r| r.map_err(|e| LedgerError::Db(format!("ImportRow -> Import conversion failed: {e}"))))
            .collect()
    }
}
