use std::sync::Arc;
use std::time::Duration;

use axum::async_trait;
use serde::Deserialize;
use serde_with::{serde_as, DurationSeconds};
use tokio::time::sleep;
use tracing::error;

use crate::domain::error::Result;
use crate::infra::db::row_models::OutboxRow;

/// Storage abstraction for reading and updating outbox records.
#[async_trait]
pub trait OutboxStore: Send + Sync {
    async fn fetch_pending(&self, limit: i64) -> Result<Vec<OutboxRow>>;
    async fn mark_published(&self, id: i32) -> Result<()>;
    async fn mark_failed(&self, id: i32, error: String) -> Result<()>;
}

/// Abstraction over the message broker.
#[async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish_event(&self, event: &OutboxRow) -> Result<()>;
}

/// Simple configuration for the outbox worker.
#[serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct OutboxWorkerConfig {
    pub batch_size: i64,
    #[serde_as(as = "DurationSeconds<u64>")]
    pub poll_interval: Duration,
}

/// Outbox worker that periodically polls the outbox table and
/// publishes pending events to the message broker.
pub struct OutboxWorker<S, P>
where
    S: OutboxStore,
    P: EventPublisher,
{
    store: Arc<S>,
    publisher: Arc<P>,
    cfg: OutboxWorkerConfig,
}

impl<S, P> OutboxWorker<S, P>
where
    S: OutboxStore,
    P: EventPublisher,
{
    pub fn new(store: Arc<S>, publisher: Arc<P>, cfg: OutboxWorkerConfig) -> Self {
        Self {
            store,
            publisher,
            cfg,
        }
    }

    pub async fn run(&self) -> Result<()> {
        loop {
            if let Err(e) = self.tick().await {
                error!("outbox worker tick error: {e:?}");
                sleep(Duration::from_secs(1)).await;
            }
        }
    }

    /// Single tick: fetch pending events and process them.
    async fn tick(&self) -> Result<()> {
        let events = self.store.fetch_pending(self.cfg.batch_size).await?;

        if events.is_empty() {
            sleep(self.cfg.poll_interval).await;
            return Ok(());
        }

        for ev in events {
            match self.publisher.publish_event(&ev).await {
                Ok(()) => {
                    self.store.mark_published(ev.id).await?;
                }
                Err(err) => {
                    error!("failed to publish outbox event_id={}: {:?}", ev.id, err);
                    self.store.mark_failed(ev.id, format!("{err:?}")).await?;
                }
            }
        }

        Ok(())
    }
}
