use crate::{
    auth_core::{
        errors::AuthError,
        models::{RefreshBlockReason, Uid},
        ports::RevocationCache,
    },
    config::RedisConfig,
};
use async_trait::async_trait;
use cache::{Cache, CacheConfig, CacheError, RedisCacheConfig};
use std::{sync::Arc, time::Duration};

#[derive(Clone)]
pub struct RedisCache {
    cache: Arc<Cache>,
    skew_secs: i64,
    ns: &'static str,
}

impl RedisCache {
    pub async fn new(cfg: RedisConfig) -> Result<Self, AuthError> {
        let cache_config = CacheConfig::Redis(RedisCacheConfig {
            url: cfg.url,
            max_size: cfg.max_size,
        });
        let cache = Cache::new(&cache_config).await?;
        Ok(Self {
            cache,
            skew_secs: cfg.skew_secs,
            ns: "auth",
        })
    }

    #[inline]
    fn key_refresh(&self, hash_b64: &str) -> String {
        format!("{}:bl:refresh:{}", self.ns, hash_b64)
    }

    #[inline]
    fn key_session(&self, session_id: Uid) -> String {
        format!("{}:bl:session:{}", self.ns, session_id)
    }
}

#[async_trait]
impl RevocationCache for RedisCache {
    async fn check_refresh(
        &self,
        session_id: Uid,
        token_hash_b64: &str,
    ) -> Result<Option<RefreshBlockReason>, AuthError> {
        let k_refresh = self.key_refresh(token_hash_b64);
        let k_session = self.key_session(session_id);

        let vals: Vec<Option<String>> = self.cache.get_many(&[&k_refresh, &k_session]).await?;
        if let Some(v) = vals.get(0).and_then(|x| x.as_ref()) {
            if v == "rotated" {
                return Ok(Some(RefreshBlockReason::Rotated));
            }
        }
        if let Some(v) = vals.get(1).and_then(|x| x.as_ref()) {
            if v == "revoked" {
                return Ok(Some(RefreshBlockReason::RevokedSession));
            }
        }
        Ok(None)
    }

    async fn mark_refresh_rotated(
        &self,
        token_hash_b64: &str,
        seconds_left: i64,
    ) -> Result<(), AuthError> {
        if seconds_left <= 0 {
            return Ok(());
        }
        let ttl = (seconds_left + self.skew_secs).max(1) as u64;
        let key = self.key_refresh(token_hash_b64);
        self.cache.insert_with_expiry(&key, "rotated", Duration::from_secs(ttl)).await?;
        Ok(())
    }

    async fn revoke_all_for_session(
        &self,
        session_id: Uid,
        session_ttl_secs: i64,
    ) -> Result<(), AuthError> {
        if session_ttl_secs <= 0 {
            return Ok(());
        }
        let ttl = (session_ttl_secs + self.skew_secs).max(1) as u64;
        let key = self.key_session(session_id);
        self.cache.insert_with_expiry(&key, &"revoked", Duration::from_secs(ttl)).await?;
        Ok(())
    }
}

impl From<CacheError> for AuthError {
    fn from(e: CacheError) -> Self {
        Self::Storage(format!("cache: {e}"))
    }
}
