pub mod config;
pub mod event_publisher;
pub mod outbox;
pub mod outbox_pg_store;
pub mod rabbitmq;

use std::sync::Arc;

use event_publisher::WorkerEventPublisher;
use outbox::OutboxWorker;
use outbox_pg_store::PgOutboxStore;
use rabbitmq::{LedgerMsg, PublishRequest, rabbitmq_publisher::RabbitmqPublisher};
use tokio::sync::mpsc;
use tracing::error;

use crate::{infra::db::make_pool, worker::config::WorkerConfig};

pub async fn start_background_workers(worker_cfg: WorkerConfig) -> anyhow::Result<()> {
    let pg = make_pool(worker_cfg.db.url.as_str(), worker_cfg.db.max_connections, worker_cfg.db.timeout).await?;
    let outbox_store = Arc::new(PgOutboxStore::new(pg));

    let (tx, rx) = mpsc::unbounded_channel::<PublishRequest<LedgerMsg>>();

    let mut rabbit_publisher = RabbitmqPublisher::new(worker_cfg.rabbitmq, rx);

    let rabbit = async move {
        if let Err(err) = rabbit_publisher.publish_to_rabbitmq().await {
            error!("RabbitMQ publisher crashed: {err:?}");
        }
    };

    let event_publisher = Arc::new(WorkerEventPublisher::new(tx));

    let worker = OutboxWorker::new(outbox_store, event_publisher, worker_cfg.outbox_worker);

    let worker = async move {
        if let Err(err) = worker.run().await {
            error!("Outbox worker crashed: {err:?}");
        }
    };

    // graceful shutdown
    tokio::select! {
        _ = rabbit => {},
        _ = worker => {},
    }

    Ok(())
}
