#![cfg(test)]

use std::sync::Arc;

use axum::async_trait;
use tokio::net::TcpListener;
use uuid::Uuid;

use crate::{
    domain::{
        error::Result,
        models::{exchange::ExchangeId, import::Import, transaction::Transaction, utils::ParseContext},
        ports::{ImportQueryRepository, TransactionQueryRepository},
        services::ExchangeService,
    },
    infra::db::row_models::TransactionRow,
    routes::{AppState, ExchangeRegistry, build_router},
};

struct NoopExchange;

#[async_trait]
impl ExchangeService for NoopExchange {
    fn id(&self) -> ExchangeId {
        ExchangeId::Mexc
    }

    async fn parse_csv(&self, _reader: Box<dyn futures::io::AsyncRead + Send + Unpin>, _ctx: ParseContext) -> Result<()> {
        Ok(())
    }

    async fn import_api(&self) -> Result<()> {
        Ok(())
    }
}

struct NoopImportRepo;

#[async_trait]
impl ImportQueryRepository for NoopImportRepo {
    async fn get(&self, _id: Uuid) -> Result<Option<Import>> {
        Ok(None)
    }

    async fn list_for_tenant(&self, _tenant_id: Uuid, _limit: i64, _offset: i64) -> Result<Vec<Import>> {
        Ok(vec![])
    }
}

struct NoopTxRepo;

#[async_trait]
impl TransactionQueryRepository for NoopTxRepo {
    async fn list_by_import(&self, _import_id: Uuid) -> Result<Vec<Transaction>> {
        Ok(vec![])
    }

    async fn list_by_tenant_import(&self, _tenant_id: Uuid, _import_id: Uuid) -> Result<Vec<TransactionRow>> {
        Ok(vec![])
    }

    async fn list_for_tenant(&self, _tenant_id: Uuid, _limit: i64, _offset: i64) -> Result<Vec<Transaction>> {
        Ok(vec![])
    }
}

#[tokio::test]
#[ignore = "manual e2e"]
async fn e2e_smoke_health_endpoint() {
    let mut registry = ExchangeRegistry::new();
    registry.insert(ExchangeId::Mexc, Box::new(NoopExchange));

    let app = build_router(AppState {
        registry: Arc::new(registry),
        import_query_repo: Arc::new(NoopImportRepo),
        transaction_query_repo: Arc::new(NoopTxRepo),
    });

    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind test port");
    let addr = listener.local_addr().expect("local addr");

    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    let server = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                let _ = rx.await;
            })
            .await
            .expect("server should run");
    });

    let client = reqwest::Client::new();
    let res = client.get(format!("http://{addr}/health")).send().await.expect("health request should succeed");
    assert_eq!(res.status(), reqwest::StatusCode::OK);

    tx.send(()).expect("shutdown signal");
    server.await.expect("server task join");
}

#[tokio::test]
#[ignore = "manual e2e"]
async fn e2e_smoke_supported_exchanges_endpoint() {
    let mut registry = ExchangeRegistry::new();
    registry.insert(ExchangeId::Mexc, Box::new(NoopExchange));

    let app = build_router(AppState {
        registry: Arc::new(registry),
        import_query_repo: Arc::new(NoopImportRepo),
        transaction_query_repo: Arc::new(NoopTxRepo),
    });

    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind test port");
    let addr = listener.local_addr().expect("local addr");

    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    let server = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                let _ = rx.await;
            })
            .await
            .expect("server should run");
    });

    let client = reqwest::Client::new();
    let res = client.get(format!("http://{addr}/v1/exchanges/supported")).send().await.expect("request should succeed");
    assert_eq!(res.status(), reqwest::StatusCode::OK);

    let body: serde_json::Value = res.json().await.expect("json response");
    assert_eq!(body["exchanges"], serde_json::json!(["mexc"]));

    tx.send(()).expect("shutdown signal");
    server.await.expect("server task join");
}
