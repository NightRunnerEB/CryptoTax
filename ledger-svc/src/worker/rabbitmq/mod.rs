use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod config;
pub mod rabbitmq;
pub mod rabbitmq_client;
pub mod rabbitmq_publisher;

/// Message abstraction for RabbitMQ publisher
pub trait OutgoingMessage: Serialize + Send + Sync + Debug + 'static {
    /// Context ID for tracing
    fn context_id(&self) -> Option<String> {
        None
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum LedgerMsg {
    ImportCreated(ImportCompleted),
}

impl OutgoingMessage for LedgerMsg {
    fn context_id(&self) -> Option<String> {
        match self {
            LedgerMsg::ImportCreated(ev) => Some(ev.event_id.to_string()),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ImportCompletedPayload {
    pub tenant_id: Uuid,
    pub import_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportCompleted {
    pub event_id: Uuid,
    pub tenant_id: Uuid,
    pub import_id: Uuid,
}
