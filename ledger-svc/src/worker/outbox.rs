use std::sync::Arc;
use std::time::Duration;

use axum::async_trait;
use serde::Deserialize;
use serde_with::{DurationSeconds, serde_as};
use tokio::time::sleep;
use tracing::error;

use crate::domain::error::Result;

/// Storage abstraction for reading and updating outbox records.
#[async_trait]
pub trait OutboxStore: Send + Sync {
    async fn process_pending(&self, limit: i64, max_attempts: i32, publisher: &dyn EventPublisher) -> Result<i64>;
}

#[derive(Debug)]
pub enum PublishError {
    Transient(String),
    Permanent(String),
}

/// Abstraction over the message broker.
#[async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish_event(&self, event: &crate::infra::db::row_models::OutboxRow) -> std::result::Result<(), PublishError>;
}

/// Simple configuration for the outbox worker.
#[serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct OutboxWorkerConfig {
    pub batch_size: i64,
    #[serde(default = "OutboxWorkerConfig::default_max_attempts")]
    pub max_attempts: i32,
    #[serde_as(as = "DurationSeconds<u64>")]
    pub poll_interval: Duration,
}

impl OutboxWorkerConfig {
    fn default_max_attempts() -> i32 {
        10
    }
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
        let processed = self.store.process_pending(self.cfg.batch_size, self.cfg.max_attempts, self.publisher.as_ref()).await?;
        if processed == 0 {
            sleep(self.cfg.poll_interval).await;
        }
        Ok(())
    }
}
