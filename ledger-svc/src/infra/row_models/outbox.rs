use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use std::{fmt, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutboxStatus {
    Pending,
    Published,
    Failed,
}

impl fmt::Display for OutboxStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            OutboxStatus::Pending => "pending",
            OutboxStatus::Published => "published",
            OutboxStatus::Failed => "failed",
        })
    }
}

impl FromStr for OutboxStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(OutboxStatus::Pending),
            "published" => Ok(OutboxStatus::Published),
            "failed" => Ok(OutboxStatus::Failed),
            other => Err(format!("unknown OutboxStatus: {other}")),
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct OutboxRow {
    pub id: i32,
    pub event_id: Uuid,
    pub tenant_id: Uuid,
    pub aggregate_type: String,
    pub aggregate_id: Uuid,
    pub event_type: String,
    pub event_version: i32,
    pub payload: serde_json::Value,
    pub headers: Option<serde_json::Value>,
    pub status: String,
    pub attempts: i32,
    pub last_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
}
