use std::sync::Arc;

use anyhow::Result;
use axum::{Router, routing::{post, get}};
use base64ct::{Base64Url, Encoding};
use serde::Deserialize;

use crate::{
    auth_core::AuthUseCases,
    config::AppConfig,
    db::make_pool,
    infra::{
        PepperSet, SmtpMailer,
        jwt_issuer_rs256::JwtIssuerRs,
        password_hasher_argon2::{Argon2Hasher, KdfParams},
        redis::RedisCache,
        refresh_factory::RefreshFactory,
        repos::{PgEmailVerificationRepo, PgRefreshRepo, PgSessionRepo, PgUserRepo},
    },
    routes::handlers::*,
};

pub mod extractors;
pub mod handlers;

pub type UC = AuthUseCases<
    PgUserRepo,
    PgSessionRepo,
    PgRefreshRepo,
    Argon2Hasher,
    JwtIssuerRs,
    RefreshFactory,
    RedisCache,
    PgEmailVerificationRepo,
    SmtpMailer,
>;

pub type AppState = Arc<UC>;

pub async fn build_state(cfg: &AppConfig) -> Result<AppState> {
    let pg = make_pool(cfg.db.url.as_str(), cfg.db.max_connections, cfg.db.timeout).await?;
    let cache = RedisCache::new(cfg.cache.clone()).await?;

    let users = PgUserRepo::new(pg.clone());
    let sessions = PgSessionRepo::new(pg.clone());
    let refresh = PgRefreshRepo::new(pg.clone());
    let email_verification = PgEmailVerificationRepo::new(pg.clone());
    let refresh_factory = RefreshFactory::new(cfg.refresh.clone());
    let mailer = SmtpMailer::new(cfg.mailer.clone())?;

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
        access_ttl: cfg.jwt.access_ttl_secs,
        refresh_ttl: cfg.refresh.ttl_secs,
        dummy_password_hash: cfg.dummy_password_hash.clone(),
    };

    Ok(Arc::new(uc))
}

/// ----- DTOs -----
#[derive(Deserialize)]
pub struct RegisterReq {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginReq {
    pub email: String,
    pub password: String,
    pub ip: Option<String>,
    pub ua: Option<String>,
}

#[derive(Deserialize)]
pub struct RefreshReq {
    pub refresh_token: String,
}

#[derive(Deserialize)]
pub struct VerifyEmailReq {
    pub token: String,
}

/// ----- Router -----
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/auth/register", post(register_handler))
        .route("/auth/login", post(login_handler))
        .route("/auth/refresh", post(refresh_handler))
        .route("/auth/logout", post(logout_handler))
        .route("/auth/verify", get(verify_email_handler))
        .route("/health", get(health_handler))
        .with_state(state)
}
