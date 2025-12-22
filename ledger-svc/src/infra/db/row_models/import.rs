use std::str::FromStr;

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use crate::domain::models::import::{Import, ImportStatus};

#[derive(Debug, Clone, FromRow)]
pub struct ImportRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub source: String,
    pub file_name: Option<String>,
    pub status: String,
    pub total_count: i32,
    pub error_summary: Option<String>,
    pub error_details: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl TryFrom<ImportRow> for Import {
    type Error = String;

    fn try_from(row: ImportRow) -> Result<Self, Self::Error> {
        Ok(Import {
            id: row.id,
            tenant_id: row.tenant_id,
            source: row.source,
            file_name: row.file_name,
            status: ImportStatus::from_str(&row.status)?,
            total_count: row.total_count,
            error_summary: row.error_summary,
            created_at: row.created_at,
            completed_at: row.completed_at,
        })
    }
}

impl From<&Import> for ImportRow {
    fn from(import: &Import) -> Self {
        ImportRow {
            id: import.id,
            tenant_id: import.tenant_id,
            source: import.source.clone(),
            file_name: import.file_name.clone(),
            status: import.status.to_string(),
            total_count: import.total_count,
            error_summary: import.error_summary.clone(),
            error_details: None,
            created_at: import.created_at,
            completed_at: import.completed_at,
        }
    }
}
