use std::sync::Arc;
use std::time::Duration;

use axum::async_trait;
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

/// Abstraction over the message broker (Kafka/Rabbit/etc).
#[async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish_event(&self, event: &OutboxRow) -> Result<()>;
}

/// Simple configuration for the outbox worker.
pub struct OutboxWorkerConfig {
    pub batch_size: i64,
    pub poll_interval: Duration,
}

impl Default for OutboxWorkerConfig {
    fn default() -> Self {
        Self {
            batch_size: 100,
            poll_interval: Duration::from_millis(500),
        }
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

    // /// Run the worker loop until the shutdown future completes.
    // pub async fn run<F>(&self, mut shutdown: F) -> Result<()>
    // where
    //     F: std::future::Future<Output = ()> + Send + 'static,
    // {
    //     loop {
    //         tokio::select! {
    //             _ = &mut shutdown => {
    //                 // Graceful shutdown requested
    //                 break;
    //             }
    //             res = self.tick() => {
    //                 // We do not break on tick errors; we just log and continue.
    //                 if let Err(e) = res {
    //                     tracing::error!("outbox worker tick error: {e:?}");
    //                     // Backoff a bit to avoid tight error loop.
    //                     sleep(Duration::from_secs(1)).await;
    //                 }
    //             }
    //         }
    //     }

    //     Ok(())
    // }

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
                    // We intentionally keep the original error in logs
                    // and store only string summary in the DB.
                    error!("failed to publish outbox event id={}: {:?}", ev.id, err);
                    self.store.mark_failed(ev.id, format!("{err:?}")).await?;
                }
            }
        }

        Ok(())
    }
}
