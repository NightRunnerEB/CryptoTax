pub mod extractors;
pub mod handlers;

use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use axum::{
    Router,
    routing::{get, post},
};

use crate::{
    application::exchanges::{mexc::MexcService, okx::OkxService},
    config::{AppConfig, load_exchange_cfg},
    domain::{models::exchange::ExchangeId, services::ExchangeService},
    infra::db::{make_pool, pq::tx_repo::PgTxRepository},
    routes::handlers::{health_handler, mexc_csv_handler},
};

pub struct ExchangeRegistry {
    exchanges: HashMap<ExchangeId, Box<dyn ExchangeService>>,
}

impl ExchangeRegistry {
    pub fn new() -> Self {
        Self {
            exchanges: HashMap::new(),
        }
    }

    pub fn get(&self, id: ExchangeId) -> Option<&Box<dyn ExchangeService>> {
        self.exchanges.get(&id)
    }

    pub fn insert(&mut self, id: ExchangeId, svc: Box<dyn ExchangeService>) {
        self.exchanges.insert(id, svc);
    }
}

#[derive(Clone)]
pub struct AppState {
    pub registry: Arc<ExchangeRegistry>,
}

pub async fn build_state(cfg: &AppConfig) -> Result<AppState> {
    let mut registry = ExchangeRegistry::new();
    use ExchangeId::*;

    let pg = make_pool(cfg.infra.db.url.as_str(), cfg.infra.db.max_connections, cfg.infra.db.timeout).await?;
    let tx_repo = PgTxRepository::new(pg);

    // Mexc
    if let Some(mexc_cfg) = cfg.exchange_cfg_paths.get(&Mexc) {
        let cfg = load_exchange_cfg(mexc_cfg)?;
        let mexc = Box::new(MexcService::new(tx_repo.clone(), cfg));
        registry.insert(Mexc, mexc);
    };

    // Okx
    if let Some(okx_cfg) = cfg.exchange_cfg_paths.get(&Okx) {
        let cfg = load_exchange_cfg(okx_cfg)?;
        let okx = Box::new(OkxService::new(tx_repo.clone(), cfg));
        registry.insert(Okx, okx);
    };

    let app_state = AppState {
        registry: Arc::new(registry),
    };

    Ok(app_state)
}

/// ----- Router -----
pub fn build_router(state: AppState) -> Router {
    Router::new().route("/health", get(health_handler)).route("/mexc/csv", post(mexc_csv_handler)).with_state(state)
}
