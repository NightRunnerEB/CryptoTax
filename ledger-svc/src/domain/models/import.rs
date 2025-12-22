use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Import {
    pub id: Uuid,
    pub tenant_id: Uuid,

    pub source: String,
    pub file_name: Option<String>,

    pub status: ImportStatus,
    pub total_count: i32,

    pub error_summary: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImportStatus {
    Processing,
    Completed,
    Failed,
    RolledBack,
}

impl fmt::Display for ImportStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ImportStatus::Processing => "processing",
            ImportStatus::Completed => "completed",
            ImportStatus::Failed => "failed",
            ImportStatus::RolledBack => "rolledBack",
        };
        f.write_str(s)
    }
}

impl FromStr for ImportStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "processing" => Ok(ImportStatus::Processing),
            "completed" => Ok(ImportStatus::Completed),
            "failed" => Ok(ImportStatus::Failed),
            "rolledBack" => Ok(ImportStatus::RolledBack),
            other => Err(format!("unknown ImportStatus: {other}")),
        }
    }
}
