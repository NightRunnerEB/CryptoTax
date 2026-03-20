#[cfg(test)]
mod e2e_smoke;
pub mod extractors;
pub mod handlers;

use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use axum::{
    Router,
    http::{Method, header},
    routing::{get, post},
};
use tower_http::cors::CorsLayer;

use crate::{
    application::exchanges::{mexc::MexcService, okx::OkxService},
    config::{AppConfig, load_exchange_cfg},
    domain::{
        models::exchange::ExchangeId,
        ports::{ImportQueryRepository, TransactionQueryRepository},
        services::ExchangeService,
    },
    infra::db::{
        make_pool,
        pq::{import_repo::PgImportRepository, tx_repo::PgTransactionQueryRepository, uow::PgImportUnitOfWorkFactory},
    },
    routes::handlers::{health_handler, list_import_transactions_handler, list_supported_exchanges_handler, mexc_csv_handler},
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

    pub fn list_supported(&self) -> Vec<String> {
        let mut out: Vec<String> = self.exchanges.keys().map(ToString::to_string).collect();
        out.sort_unstable();
        out
    }
}

#[derive(Clone)]
pub struct AppState {
    pub registry: Arc<ExchangeRegistry>,
    pub import_query_repo: Arc<dyn ImportQueryRepository>,
    pub transaction_query_repo: Arc<dyn TransactionQueryRepository>,
}

pub async fn build_state(cfg: &AppConfig) -> Result<AppState> {
    let mut registry = ExchangeRegistry::new();
    use ExchangeId::*;

    let pg = make_pool(cfg.infra.db.url.as_str(), cfg.infra.db.max_connections, cfg.infra.db.timeout).await?;
    let uow_factory = PgImportUnitOfWorkFactory::new(pg.clone());
    let import_repo = PgImportRepository::new(pg.clone());
    let tx_query_repo = PgTransactionQueryRepository::new(pg.clone());

    // Mexc
    if let Some(mexc_cfg) = cfg.exchange_cfg_paths.get(&Mexc) {
        let cfg = load_exchange_cfg(mexc_cfg)?;
        let mexc = Box::new(MexcService::new(uow_factory.clone(), import_repo.clone(), cfg));
        registry.insert(Mexc, mexc);
    };

    // Okx
    if let Some(okx_cfg) = cfg.exchange_cfg_paths.get(&Okx) {
        let cfg = load_exchange_cfg(okx_cfg)?;
        let okx = Box::new(OkxService::new(uow_factory.clone(), import_repo.clone(), cfg));
        registry.insert(Okx, okx);
    };

    let app_state = AppState {
        registry: Arc::new(registry),
        import_query_repo: Arc::new(import_repo),
        transaction_query_repo: Arc::new(tx_query_repo),
    };

    Ok(app_state)
}

fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        // TODO: replace localhost frontend origins with your gateway/web app origin.
        .allow_origin([
            header::HeaderValue::from_static("http://localhost:5173"),
            header::HeaderValue::from_static("http://127.0.0.1:5173"),
        ])
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::PATCH, Method::DELETE, Method::OPTIONS])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::ACCEPT,
            header::HeaderName::from_static("x-tenant-id"),
            header::HeaderName::from_static("x-user-id"),
            header::HeaderName::from_static("x-roles"),
        ])
}

/// ----- Router -----
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/v1/exchanges/supported", get(list_supported_exchanges_handler))
        .route("/mexc/csv", post(mexc_csv_handler))
        .route("/v1/tenants/:tenant_id/imports/:import_id/transactions", get(list_import_transactions_handler))
        .with_state(state)
        .layer(cors_layer())
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    };

    use axum::{
        async_trait,
        body::Body,
        http::{Request, StatusCode},
    };
    use futures::io::AsyncRead;
    use http_body_util::BodyExt;
    use tower::util::ServiceExt;
    use uuid::Uuid;

    use super::{AppState, ExchangeRegistry, build_router};
    use crate::{
        domain::{
            error::Result,
            models::{
                exchange::ExchangeId,
                import::{Import, ImportStatus},
            },
            ports::{ImportQueryRepository, TransactionQueryRepository},
            services::ExchangeService,
        },
        infra::db::row_models::TransactionRow,
    };

    fn take_expected<T>(slot: &Mutex<Option<Result<T>>>, name: &str) -> Result<T> {
        slot.lock().expect("mutex poisoned").take().unwrap_or_else(|| panic!("unexpected call: {name}"))
    }

    struct FakeExchangeService {
        id: ExchangeId,
        parse_result: Mutex<Option<Result<()>>>,
        parse_calls: Arc<AtomicUsize>,
    }

    impl FakeExchangeService {
        fn new(id: ExchangeId, parse_result: Result<()>) -> Self {
            Self {
                id,
                parse_result: Mutex::new(Some(parse_result)),
                parse_calls: Arc::new(AtomicUsize::new(0)),
            }
        }
    }

    #[async_trait]
    impl ExchangeService for FakeExchangeService {
        fn id(&self) -> ExchangeId {
            self.id
        }

        async fn parse_csv(
            &self, _reader: Box<dyn AsyncRead + Send + Unpin>, _ctx: crate::domain::models::utils::ParseContext,
        ) -> Result<()> {
            self.parse_calls.fetch_add(1, Ordering::Relaxed);
            take_expected(&self.parse_result, "ExchangeService.parse_csv")
        }

        async fn import_api(&self) -> Result<()> {
            Ok(())
        }
    }

    #[derive(Default)]
    struct FakeImportQueryRepo {
        get_result: Mutex<Option<Result<Option<Import>>>>,
    }

    #[async_trait]
    impl ImportQueryRepository for FakeImportQueryRepo {
        async fn get(&self, _id: Uuid) -> Result<Option<Import>> {
            take_expected(&self.get_result, "ImportQueryRepository.get")
        }

        async fn list_for_tenant(&self, _tenant_id: Uuid, _limit: i64, _offset: i64) -> Result<Vec<Import>> {
            Ok(vec![])
        }
    }

    #[derive(Default)]
    struct FakeTxQueryRepo {
        list_by_tenant_import_result: Mutex<Option<Result<Vec<TransactionRow>>>>,
    }

    #[async_trait]
    impl TransactionQueryRepository for FakeTxQueryRepo {
        async fn list_by_import(&self, _import_id: Uuid) -> Result<Vec<crate::domain::models::transaction::Transaction>> {
            Ok(vec![])
        }

        async fn list_by_tenant_import(&self, _tenant_id: Uuid, _import_id: Uuid) -> Result<Vec<TransactionRow>> {
            take_expected(&self.list_by_tenant_import_result, "TransactionQueryRepository.list_by_tenant_import")
        }

        async fn list_for_tenant(
            &self, _tenant_id: Uuid, _limit: i64, _offset: i64,
        ) -> Result<Vec<crate::domain::models::transaction::Transaction>> {
            Ok(vec![])
        }
    }

    fn import_for(tenant_id: Uuid) -> Import {
        Import {
            id: Uuid::new_v4(),
            tenant_id,
            source: "mexc".to_string(),
            file_name: Some("a.csv".to_string()),
            status: ImportStatus::Completed,
            total_count: 2,
            error_summary: None,
            created_at: chrono::Utc::now(),
            completed_at: Some(chrono::Utc::now()),
        }
    }

    fn sample_tx_row(tenant_id: Uuid, import_id: Uuid) -> TransactionRow {
        TransactionRow {
            id: Uuid::new_v4(),
            tenant_id,
            source: "MEXC".to_string(),
            time_utc: chrono::Utc::now(),
            kind: "Spot".to_string(),
            in_money: Some(serde_json::json!({"symbol":"BTC","amount":"0.1"})),
            out_money: Some(serde_json::json!({"symbol":"USDT","amount":"3000"})),
            fee_money: None,
            contract_symbol: None,
            derivative_kind: None,
            position_id: None,
            order_id: Some("order-1".to_string()),
            tx_hash: None,
            note: None,
            import_id,
            tx_fingerprint: "fp-1".to_string(),
        }
    }

    fn app_state_with(registry: ExchangeRegistry, import_repo: FakeImportQueryRepo, tx_repo: FakeTxQueryRepo) -> AppState {
        AppState {
            registry: Arc::new(registry),
            import_query_repo: Arc::new(import_repo),
            transaction_query_repo: Arc::new(tx_repo),
        }
    }

    #[tokio::test]
    async fn list_supported_exchanges_returns_sorted_list() {
        let mut registry = ExchangeRegistry::new();
        registry.insert(ExchangeId::Okx, Box::new(FakeExchangeService::new(ExchangeId::Okx, Ok(()))));
        registry.insert(ExchangeId::Mexc, Box::new(FakeExchangeService::new(ExchangeId::Mexc, Ok(()))));

        let app = build_router(app_state_with(registry, FakeImportQueryRepo::default(), FakeTxQueryRepo::default()));

        let res = app
            .oneshot(Request::builder().uri("/v1/exchanges/supported").body(Body::empty()).expect("request build"))
            .await
            .expect("request handled");
        assert_eq!(res.status(), StatusCode::OK);

        let body = res.into_body().collect().await.expect("body read").to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).expect("json parse");
        assert_eq!(json["exchanges"], serde_json::json!(["mexc", "okx"]));
    }

    #[tokio::test]
    async fn list_import_transactions_invalid_tenant_uuid_returns_400() {
        let app =
            build_router(app_state_with(ExchangeRegistry::new(), FakeImportQueryRepo::default(), FakeTxQueryRepo::default()));

        let res = app
            .oneshot(
                Request::builder()
                    .uri("/v1/tenants/not-uuid/imports/550e8400-e29b-41d4-a716-446655440000/transactions")
                    .body(Body::empty())
                    .expect("request build"),
            )
            .await
            .expect("request handled");
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn list_import_transactions_import_not_found_returns_404() {
        let import_repo = FakeImportQueryRepo {
            get_result: Mutex::new(Some(Ok(None))),
        };
        let app = build_router(app_state_with(ExchangeRegistry::new(), import_repo, FakeTxQueryRepo::default()));

        let tenant = Uuid::new_v4();
        let import = Uuid::new_v4();
        let res = app
            .oneshot(
                Request::builder()
                    .uri(format!("/v1/tenants/{tenant}/imports/{import}/transactions"))
                    .body(Body::empty())
                    .expect("request build"),
            )
            .await
            .expect("request handled");
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn list_import_transactions_tenant_mismatch_returns_404() {
        let import_repo = FakeImportQueryRepo {
            get_result: Mutex::new(Some(Ok(Some(import_for(Uuid::new_v4()))))),
        };
        let app = build_router(app_state_with(ExchangeRegistry::new(), import_repo, FakeTxQueryRepo::default()));

        let tenant = Uuid::new_v4();
        let import = Uuid::new_v4();
        let res = app
            .oneshot(
                Request::builder()
                    .uri(format!("/v1/tenants/{tenant}/imports/{import}/transactions"))
                    .body(Body::empty())
                    .expect("request build"),
            )
            .await
            .expect("request handled");
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn list_import_transactions_success_returns_rows() {
        let tenant = Uuid::new_v4();
        let mut import = import_for(tenant);
        import.id = Uuid::new_v4();

        let import_repo = FakeImportQueryRepo {
            get_result: Mutex::new(Some(Ok(Some(import.clone())))),
        };
        let tx_repo = FakeTxQueryRepo {
            list_by_tenant_import_result: Mutex::new(Some(Ok(vec![sample_tx_row(tenant, import.id)]))),
        };
        let app = build_router(app_state_with(ExchangeRegistry::new(), import_repo, tx_repo));

        let res = app
            .oneshot(
                Request::builder()
                    .uri(format!("/v1/tenants/{tenant}/imports/{}/transactions", import.id))
                    .body(Body::empty())
                    .expect("request build"),
            )
            .await
            .expect("request handled");
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn mexc_csv_missing_tenant_header_returns_403() {
        let service = FakeExchangeService::new(ExchangeId::Mexc, Ok(()));
        let parse_calls = service.parse_calls.clone();

        let mut registry = ExchangeRegistry::new();
        registry.insert(ExchangeId::Mexc, Box::new(service));

        let app = build_router(app_state_with(registry, FakeImportQueryRepo::default(), FakeTxQueryRepo::default()));

        let boundary = "XBOUNDARY";
        let body = format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"test.csv\"\r\nContent-Type: text/csv\r\n\r\na,b\n1,2\n\r\n--{boundary}--\r\n"
        );

        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/mexc/csv")
                    .header("content-type", format!("multipart/form-data; boundary={boundary}"))
                    .body(Body::from(body))
                    .expect("request build"),
            )
            .await
            .expect("request handled");

        assert_eq!(res.status(), StatusCode::FORBIDDEN);
        assert_eq!(parse_calls.load(Ordering::Relaxed), 0);
    }

    #[tokio::test]
    async fn mexc_csv_success_calls_exchange_parser() {
        let service = FakeExchangeService::new(ExchangeId::Mexc, Ok(()));
        let parse_calls = service.parse_calls.clone();

        let mut registry = ExchangeRegistry::new();
        registry.insert(ExchangeId::Mexc, Box::new(service));

        let app = build_router(app_state_with(registry, FakeImportQueryRepo::default(), FakeTxQueryRepo::default()));

        let tenant = Uuid::new_v4();
        let boundary = "XBOUNDARY";
        let body = format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"test.csv\"\r\nContent-Type: text/csv\r\n\r\na,b\n1,2\n\r\n--{boundary}--\r\n"
        );

        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/mexc/csv")
                    .header("content-type", format!("multipart/form-data; boundary={boundary}"))
                    .header("x-user-id", tenant.to_string())
                    .body(Body::from(body))
                    .expect("request build"),
            )
            .await
            .expect("request handled");

        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(parse_calls.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn health_returns_ok() {
        let app =
            build_router(app_state_with(ExchangeRegistry::new(), FakeImportQueryRepo::default(), FakeTxQueryRepo::default()));

        let res = app
            .oneshot(Request::builder().uri("/health").body(Body::empty()).expect("request build"))
            .await
            .expect("request handled");
        assert_eq!(res.status(), StatusCode::OK);
    }
}
