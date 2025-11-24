mod application;
mod config;
mod domain;
mod error;
mod infra;
mod routes;
mod worker;

use std::net::SocketAddr;

use anyhow::Result;
use axum::Router;
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    config::AppConfig,
    routes::{build_router, build_state},
};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,tower_http=info,sqlx=warn".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // let worker_cfg = OutboxWorkerConfig::default();
    // let worker = OutboxWorker::new(store, publisher, worker_cfg);

    // let shutdown = async {
    //     let _ = signal::ctrl_c().await;
    // };

    // tokio::spawn(async move {
    //     if let Err(e) = worker.run(shutdown).await {
    //         tracing::error!("outbox worker terminated with error: {e:?}");
    //     }
    // });

    let cfg = AppConfig::build_config("./config.yaml")?;
    let state = build_state(&cfg).await?;

    let app: Router = build_router(state);
    let addr: SocketAddr = cfg.infra.server.addr.parse()?;
    let listener = TcpListener::bind(addr).await?;

    tracing::info!(%addr, "listening");
    axum::serve(listener, app).await.expect("Server crashed");
    Ok(())
}
