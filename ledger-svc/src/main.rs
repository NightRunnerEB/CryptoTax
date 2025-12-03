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
    worker::config::WorkerConfig,
};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info, debug, tower_http=info,sqlx=warn".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cfg = AppConfig::build_config("./config.yaml")?;
    let state = build_state(&cfg).await?;

    let app: Router = build_router(state);
    let addr: SocketAddr = cfg.infra.server.addr.parse()?;
    let listener = TcpListener::bind(addr).await?;

    let worker_cfg = WorkerConfig::build_config("./worker_config.yaml")?;

    tokio::spawn(async move {
        if let Err(err) = worker::start_background_workers(worker_cfg).await {
            tracing::error!("background workers crashed: {err:?}");
        }
    });

    tracing::info!(%addr, "listening");
    axum::serve(listener, app).await.expect("Server crashed");
    Ok(())
}
