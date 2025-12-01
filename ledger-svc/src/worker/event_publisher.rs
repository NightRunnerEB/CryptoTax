use axum::async_trait;
use tokio::sync::mpsc::UnboundedSender;
use tracing::error;

use super::outbox::EventPublisher;
use crate::{
    domain::error::{LedgerError, Result},
    infra::db::row_models::OutboxRow,
    worker::rabbitmq::{ImportCompleted, ImportCompletedPayload, LedgerMsg},
};

pub struct WorkerEventPublisher {
    sender: UnboundedSender<LedgerMsg>,
}

impl WorkerEventPublisher {
    pub fn new(sender: UnboundedSender<LedgerMsg>) -> Self {
        Self {
            sender,
        }
    }
}

#[async_trait]
impl EventPublisher for WorkerEventPublisher {
    async fn publish_event(&self, ev: &OutboxRow) -> Result<()> {
        let payload: ImportCompletedPayload = serde_json::from_value(ev.payload.clone()).map_err(|e| {
            error!(
                outbox_id = ev.id,
                error = %e,
                "Failed to deserialize ImportCompletedPayload from outbox payload"
            );
            LedgerError::Db(format!("outbox.deserialize_payload(id={}): {e}", ev.id))
        })?;

        let msg = LedgerMsg::ImportCreated(ImportCompleted {
            event_id: ev.event_id,
            tenant_id: payload.tenant_id,
            import_id: payload.import_id,
        });

        self.sender.send(msg).map_err(|e| {
            error!(
                event_id = ev.id,
                error = %e,
                "Failed to send LedgerMsg to RabbitMQ channel"
            );
            LedgerError::Db(format!("outbox.publish_event.send(id={}): {e}", ev.id))
        })?;

        Ok(())
    }
}
