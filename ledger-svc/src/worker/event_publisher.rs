use axum::async_trait;
use tokio::sync::{mpsc::UnboundedSender, oneshot};
use tracing::error;

use super::outbox::{EventPublisher, PublishError};
use crate::{
    infra::db::row_models::OutboxRow,
    worker::rabbitmq::{ImportCompleted, ImportCompletedPayload, LedgerMsg, PublishRequest},
};

pub struct WorkerEventPublisher {
    sender: UnboundedSender<PublishRequest<LedgerMsg>>,
}

impl WorkerEventPublisher {
    pub fn new(sender: UnboundedSender<PublishRequest<LedgerMsg>>) -> Self {
        Self {
            sender,
        }
    }
}

#[async_trait]
impl EventPublisher for WorkerEventPublisher {
    async fn publish_event(&self, ev: &OutboxRow) -> std::result::Result<(), PublishError> {
        let payload: ImportCompletedPayload = serde_json::from_value(ev.payload.clone()).map_err(|e| {
            error!(
                outbox_id = ev.id,
                error = %e,
                "Failed to deserialize ImportCompletedPayload from outbox payload"
            );
            PublishError::Permanent(format!("outbox.deserialize_payload(id={}): {e}", ev.id))
        })?;

        let msg = LedgerMsg::ImportCompleted(ImportCompleted {
            event_id: ev.event_id,
            tenant_id: payload.tenant_id,
            import_id: payload.import_id,
        });

        let (ack_tx, ack_rx) = oneshot::channel();

        self.sender
            .send(PublishRequest {
                msg,
                ack: ack_tx,
            })
            .map_err(|e| {
                error!(
                    event_id = ev.id,
                    error = %e,
                    "Failed to send LedgerMsg to RabbitMQ channel"
                );
                PublishError::Transient(format!("outbox.publish_event.send(id={}): {e}", ev.id))
            })?;

        match ack_rx.await {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => Err(PublishError::Transient(format!("outbox.publish_event.broker(id={}): {e}", ev.id))),
            Err(e) => Err(PublishError::Transient(format!("outbox.publish_event.ack(id={}): {}", ev.id, e))),
        }
    }
}
