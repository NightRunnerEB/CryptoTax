use std::sync::Arc;

use anyhow::Result;
use cache::CacheConfig;
use serde::Deserialize;

use crate::{
    auth_core::AuthUseCases,
    config::AppConfig,
    db::make_pool,
    infra::{
        PepperSet, SesMailer, jwt_issuer_rs256::JwtIssuerRs, password_hasher_argon2::Argon2Hasher, redis::RedisCache, refresh_factory::RefreshFactory, repos::{PgEmailVerificationRepo, PgRefreshRepo, PgSessionRepo, PgUserRepo}
    },
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
    SesMailer,
>;

// pub async fn build_uc(cfg: &AppConfig) -> Result<Arc<UC>> {
//     let pg = make_pool(cfg.db.url.as_str(), cfg.db.max_connections).await?;

//     let users = PgUserRepo::new(pg.clone());
//     let sessions = PgSessionRepo::new(pg.clone());
//     let refresh = PgRefreshRepo::new(pg.clone());
//     let email_verification = PgEmailVerificationRepo::new(pg.clone());
//     let cache = {
//     };
//     let mailer = SesMailer::new(cfg.ses).await;

//     let pepper_current = B64.decode(cfg.kdf.pepper_current_b64.as_bytes())
//         .context("decode PEPPER_CURRENT_B64")?;
//     let peppers = PepperSet::new_current_only(pepper_current);
//     let hasher = Argon2Hasher::new(
//         KdfConfig {
//             m_cost_kib: cfg.kdf.m_cost_kib,
//             t_cost: cfg.kdf.t_cost,
//             p_lanes: cfg.kdf.p_lanes,
//         },
//         peppers,
//     )?;

//     let keys = load_rs_keys(&cfg.jwt)?;
//     let access = JwtIssuerRs {
//         iss: cfg.jwt.issuer.clone(),
//         aud: cfg.jwt.audience.clone(),
//         leeway_secs: cfg.jwt.leeway_secs,
//         keys,
//     };

//     let refresh_factory = RefreshFactory { prefix: cfg.refresh.prefix };

//     let uc = AuthUseCases {
//         users,
//         sessions,
//         refresh,
//         hasher,
//         access,
//         mailer,
//         refresh_factory,
//         email_verification,
//         cache,


//     }
//     Ok(())
// }

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
