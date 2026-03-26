pub mod dto;
#[cfg(test)]
mod e2e_smoke;
pub mod extractors;
pub mod handlers;

use std::sync::Arc;

use anyhow::Result;
use axum::{
    Router,
    http::{Method, header},
    routing::{get, post},
};
use base64ct::{Base64Url, Encoding};
use tower_http::cors::CorsLayer;

use crate::{
    auth_core::{AuthService, AuthUseCases},
    config::AppConfig,
    db::make_pool,
    infra::{
        PepperSet, SmtpMailer, TaxSvcClient,
        jwt_issuer_rs256::JwtIssuerRs,
        password_hasher_argon2::{Argon2Hasher, KdfParams},
        redis::RedisCache,
        refresh_factory::RefreshFactory,
        repos::{PgEmailVerificationRepo, PgRefreshRepo, PgSessionRepo, PgUserRepo},
    },
    routes::handlers::*,
};

type Auth = Arc<dyn AuthService>;

#[derive(Clone)]
pub struct AppState {
    pub auth: Auth,
}

impl axum::extract::FromRef<AppState> for Auth {
    fn from_ref(state: &AppState) -> Self {
        state.auth.clone()
    }
}

pub async fn build_state(cfg: &AppConfig) -> Result<AppState> {
    let pg = make_pool(cfg.db.url.as_str(), cfg.db.max_connections, cfg.db.timeout).await?;
    let cache = RedisCache::new(cfg.cache.clone()).await?;

    let users = PgUserRepo::new(pg.clone());
    let sessions = PgSessionRepo::new(pg.clone());
    let refresh = PgRefreshRepo::new(pg.clone());
    let email_verification = PgEmailVerificationRepo::new(pg.clone());
    let refresh_factory = RefreshFactory::new(cfg.refresh.clone());
    let mailer = SmtpMailer::new(cfg.mailer.clone())?;
    let tax_profiles = TaxSvcClient::new(cfg.tax_svc.clone())?;

    let decoded_pepper: Vec<u8> = Base64Url::decode_vec(cfg.password.pepper.as_str()).unwrap();
    let peppers = PepperSet::new_current_only(decoded_pepper);
    let hasher = Argon2Hasher::new(
        KdfParams {
            m_cost_kib: cfg.password.m_cost_kib,
            t_cost: cfg.password.t_cost,
            p_lanes: cfg.password.p_lanes,
        },
        peppers,
    )?;
    let access = JwtIssuerRs::new(cfg.jwt.clone());

    let uc = AuthUseCases {
        users,
        sessions,
        refresh,
        hasher,
        access,
        mailer,
        refresh_factory,
        cache,
        email_verification,
        verify_config: cfg.verify.clone(),
        tax_profiles,
        access_ttl: cfg.jwt.access_ttl_secs,
        refresh_ttl: cfg.refresh.ttl_secs,
        dummy_password_hash: cfg.dummy_password_hash.clone(),
    };

    Ok(AppState {
        auth: Arc::new(uc),
    })
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
        .route("/auth/register", post(register_handler))
        .route("/auth/login", post(login_handler))
        .route("/auth/refresh", post(refresh_handler))
        .route("/auth/logout", post(logout_handler))
        .route("/auth/verify", get(verify_email_handler))
        .route("/health", get(health_handler))
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
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use tower::util::ServiceExt;

    use super::{AppState, build_router};
    use crate::auth_core::{
        AuthService,
        errors::{AuthError, Result},
        models::{LoginResult, RegisterTaxProfile, Session, SessionStatus, Tokens, User, UserStatus},
    };

    fn take_expected<T>(slot: &Mutex<Option<Result<T>>>, name: &str) -> Result<T> {
        slot.lock().expect("mutex poisoned").take().unwrap_or_else(|| panic!("unexpected call: {name}"))
    }

    struct FakeAuth {
        register_result: Mutex<Option<Result<()>>>,
        login_result: Mutex<Option<Result<LoginResult>>>,
        refresh_result: Mutex<Option<Result<Tokens>>>,
        logout_result: Mutex<Option<Result<()>>>,
        verify_result: Mutex<Option<Result<()>>>,

        register_calls: Arc<AtomicUsize>,
        login_calls: Arc<AtomicUsize>,
        refresh_calls: Arc<AtomicUsize>,
        logout_calls: Arc<AtomicUsize>,
        verify_calls: Arc<AtomicUsize>,
    }

    impl Default for FakeAuth {
        fn default() -> Self {
            Self {
                register_result: Mutex::new(None),
                login_result: Mutex::new(None),
                refresh_result: Mutex::new(None),
                logout_result: Mutex::new(None),
                verify_result: Mutex::new(None),
                register_calls: Arc::new(AtomicUsize::new(0)),
                login_calls: Arc::new(AtomicUsize::new(0)),
                refresh_calls: Arc::new(AtomicUsize::new(0)),
                logout_calls: Arc::new(AtomicUsize::new(0)),
                verify_calls: Arc::new(AtomicUsize::new(0)),
            }
        }
    }

    #[axum::async_trait]
    impl AuthService for FakeAuth {
        async fn register(&self, _email: &str, _pwd: &str, _tax_profile: &RegisterTaxProfile) -> Result<()> {
            self.register_calls.fetch_add(1, Ordering::Relaxed);
            take_expected(&self.register_result, "Auth.register")
        }

        async fn verify_email(&self, _token: &str) -> Result<()> {
            self.verify_calls.fetch_add(1, Ordering::Relaxed);
            take_expected(&self.verify_result, "Auth.verify_email")
        }

        async fn refresh(&self, _refresh_token: &str) -> Result<Tokens> {
            self.refresh_calls.fetch_add(1, Ordering::Relaxed);
            take_expected(&self.refresh_result, "Auth.refresh")
        }

        async fn logout(&self, _access: &str) -> Result<()> {
            self.logout_calls.fetch_add(1, Ordering::Relaxed);
            take_expected(&self.logout_result, "Auth.logout")
        }

        async fn login(&self, _email: &str, _pwd: &str, _ip: Option<String>, _ua: Option<String>) -> Result<LoginResult> {
            self.login_calls.fetch_add(1, Ordering::Relaxed);
            take_expected(&self.login_result, "Auth.login")
        }
    }

    fn dummy_login_result() -> LoginResult {
        LoginResult {
            user: User {
                id: uuid::Uuid::new_v4(),
                email: "user@example.com".to_string(),
                status: UserStatus::Active,
                created_at: chrono::Utc::now(),
            },
            session: Session {
                id: uuid::Uuid::new_v4(),
                user_id: uuid::Uuid::new_v4(),
                status: SessionStatus::Active,
                created_at: chrono::Utc::now(),
                last_seen_at: chrono::Utc::now(),
                ip: Some("127.0.0.1".to_string()),
                user_agent: Some("tests".to_string()),
            },
            tokens: Tokens {
                access_token: "access-token".to_string(),
                refresh_token: "refresh-token".to_string(),
                access_expires_in: 900,
                refresh_expires_in: 2_592_000,
            },
        }
    }

    fn dummy_tax_profile_json() -> serde_json::Value {
        serde_json::json!({
          "inn": "123456789012",
          "last_name": "Ivanov",
          "first_name": "Ivan",
          "middle_name": "",
          "jurisdiction": "RU",
          "timezone": "Europe/Moscow",
          "phone": "",
          "wallets": [],
          "tax_residency_status": "resident",
          "taxpayer_type": "individual"
        })
    }

    #[tokio::test]
    async fn health_returns_ok() {
        let auth = Arc::new(FakeAuth::default()) as Arc<dyn AuthService>;
        let app = build_router(AppState {
            auth,
        });

        let response = app
            .oneshot(Request::builder().uri("/health").body(Body::empty()).expect("request must be built"))
            .await
            .expect("request should be handled");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn register_returns_200_and_calls_service() {
        let fake = FakeAuth {
            register_result: Mutex::new(Some(Ok(()))),
            ..Default::default()
        };
        let register_calls = fake.register_calls.clone();

        let auth = Arc::new(fake) as Arc<dyn AuthService>;
        let app = build_router(AppState {
            auth,
        });

        let payload = serde_json::json!({
          "email": "user@example.com",
          "password": "W7!fPq2#Kb9@Lm4$Tx",
          "tax_profile": dummy_tax_profile_json()
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/register")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .expect("request must be built"),
            )
            .await
            .expect("request should be handled");

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(register_calls.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn login_returns_401_for_invalid_credentials() {
        let fake = FakeAuth {
            login_result: Mutex::new(Some(Err(AuthError::InvalidCredentials))),
            ..Default::default()
        };
        let login_calls = fake.login_calls.clone();

        let auth = Arc::new(fake) as Arc<dyn AuthService>;
        let app = build_router(AppState {
            auth,
        });

        let payload = serde_json::json!({
          "email": "user@example.com",
          "password": "wrong-password"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/login")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .expect("request must be built"),
            )
            .await
            .expect("request should be handled");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(login_calls.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn logout_without_bearer_returns_401_and_does_not_call_service() {
        let fake = FakeAuth {
            logout_result: Mutex::new(Some(Ok(()))),
            ..Default::default()
        };
        let logout_calls = fake.logout_calls.clone();

        let auth = Arc::new(fake) as Arc<dyn AuthService>;
        let app = build_router(AppState {
            auth,
        });

        let response = app
            .oneshot(Request::builder().method("POST").uri("/auth/logout").body(Body::empty()).expect("request must be built"))
            .await
            .expect("request should be handled");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(logout_calls.load(Ordering::Relaxed), 0, "extractor should reject request before usecase call");
    }

    #[tokio::test]
    async fn refresh_returns_tokens_body() {
        let fake = FakeAuth {
            refresh_result: Mutex::new(Some(Ok(Tokens {
                access_token: "new-access".to_string(),
                refresh_token: "new-refresh".to_string(),
                access_expires_in: 900,
                refresh_expires_in: 2_592_000,
            }))),
            ..Default::default()
        };
        let refresh_calls = fake.refresh_calls.clone();

        let auth = Arc::new(fake) as Arc<dyn AuthService>;
        let app = build_router(AppState {
            auth,
        });

        let payload = serde_json::json!({
          "refresh_token": "old-refresh-token"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/refresh")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .expect("request must be built"),
            )
            .await
            .expect("request should be handled");

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(refresh_calls.load(Ordering::Relaxed), 1);

        let body = response.into_body().collect().await.expect("body should be read").to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).expect("json must be parsed");
        assert_eq!(json["access_token"], "new-access");
        assert_eq!(json["refresh_token"], "new-refresh");
    }

    #[tokio::test]
    async fn login_returns_tokens_body_on_success() {
        let fake = FakeAuth {
            login_result: Mutex::new(Some(Ok(dummy_login_result()))),
            ..Default::default()
        };

        let auth = Arc::new(fake) as Arc<dyn AuthService>;
        let app = build_router(AppState {
            auth,
        });

        let payload = serde_json::json!({
          "email": "user@example.com",
          "password": "W7!fPq2#Kb9@Lm4$Tx"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/login")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .expect("request must be built"),
            )
            .await
            .expect("request should be handled");

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.expect("body should be read").to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).expect("json must be parsed");
        assert_eq!(json["tokens"]["access_token"], "access-token");
        assert_eq!(json["tokens"]["refresh_token"], "refresh-token");
    }
}
