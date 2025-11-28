use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod config;
pub mod rabbitmq;
pub mod rabbitmq_client;
pub mod rabbitmq_publisher;

/// Message abstraction for RabbitMQ publisher
pub trait OutgoingMessage: Serialize + Send + Sync + 'static {
    /// Context ID for tracing
    fn context_id(&self) -> Option<String> {
        None
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", content = "data", rename_all = "camelCase")]
pub enum LedgerMsg {
    ImportCreated(ImportCompleted),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportCompleted {
    pub event_id: Uuid,
    pub tenant_id: Uuid,
    pub import_id: Uuid,
}
