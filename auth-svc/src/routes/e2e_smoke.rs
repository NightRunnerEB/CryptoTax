#![cfg(test)]

use std::sync::{Arc, Mutex};

use axum::async_trait;
use tokio::net::TcpListener;

use crate::{
    auth_core::{
        AuthService,
        errors::{AuthError, Result},
        models::{LoginResult, RegisterTaxProfile, Tokens},
    },
    routes::{AppState, build_router},
};

struct E2EFakeAuth {
    login_result: Mutex<Option<Result<LoginResult>>>,
}

#[async_trait]
impl AuthService for E2EFakeAuth {
    async fn register(&self, _email: &str, _pwd: &str, _tax_profile: &RegisterTaxProfile) -> Result<()> {
        Ok(())
    }

    async fn verify_email(&self, _token: &str) -> Result<()> {
        Ok(())
    }

    async fn refresh(&self, _refresh_token: &str) -> Result<Tokens> {
        Ok(Tokens {
            access_token: "new-access".to_string(),
            refresh_token: "new-refresh".to_string(),
            access_expires_in: 900,
            refresh_expires_in: 2_592_000,
        })
    }

    async fn logout(&self, _access: &str) -> Result<()> {
        Ok(())
    }

    async fn login(&self, _email: &str, _pwd: &str, _ip: Option<String>, _ua: Option<String>) -> Result<LoginResult> {
        self.login_result.lock().expect("mutex poisoned").take().unwrap_or(Err(AuthError::InvalidCredentials))
    }
}

#[tokio::test]
#[ignore = "manual e2e"]
async fn e2e_smoke_health_endpoint() {
    let auth = Arc::new(E2EFakeAuth {
        login_result: Mutex::new(None),
    }) as Arc<dyn AuthService>;

    let app = build_router(AppState {
        auth,
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
async fn e2e_smoke_login_invalid_credentials() {
    let auth = Arc::new(E2EFakeAuth {
        login_result: Mutex::new(Some(Err(AuthError::InvalidCredentials))),
    }) as Arc<dyn AuthService>;

    let app = build_router(AppState {
        auth,
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
    let res = client
        .post(format!("http://{addr}/auth/login"))
        .json(&serde_json::json!({
            "email": "user@example.com",
            "password": "wrong"
        }))
        .send()
        .await
        .expect("login request should succeed");
    assert_eq!(res.status(), reqwest::StatusCode::UNAUTHORIZED);

    tx.send(()).expect("shutdown signal");
    server.await.expect("server task join");
}
