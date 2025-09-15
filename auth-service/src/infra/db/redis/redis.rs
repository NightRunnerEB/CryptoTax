use crate::auth_core::{errors::AuthError, models::Uid, ports::RevocationCache};
use async_trait::async_trait;
use cache::{Cache, CacheConfig, CacheError};
use std::sync::Arc;

#[derive(Clone)]
pub struct RedisCache {
    cache: Arc<Cache>,
    skew_secs: i64,
    ns: &'static str,
}

impl RedisCache {
    pub async fn new(config: CacheConfig, skew_secs: i64) -> Result<Self, AuthError> {
        let cache = Cache::new(&config).await?;
        Ok(Self {
            cache,
            skew_secs,
            ns: "auth",
        })
    }

    #[inline]
    fn session_key(&self, session_id: Uid) -> String {
        format!("{}:bl:session:{session_id}", self.ns)
    }

    #[inline]
    fn refresh_key(&self, hash_b64url: &str) -> String {
        format!("{}:bl:refresh:{hash_b64url}", self.ns)
    }
}

#[async_trait]
impl RevocationCache for RedisCache {
    async fn is_refresh_blocked(&self, session_id: Uid, hash_b64url: &str) -> bool {
        let skey = self.session_key(session_id);
        let rkey = self.refresh_key(hash_b64url);

        match self.cache.exists_many(&[&skey, &rkey]).await {
            Ok(flags) => flags.into_iter().any(|b| b),
            Err(e) => {
                // soft-fail: не валим аутентификацию из-за Redis
                tracing::warn!(?e, %session_id, "redis exists_many failed; treating as not blocked");
                false
            }
        }
    }

    async fn mark_refresh_rotated(
        &self,
        hash_b64url: &str,
        seconds_left: i64,
    ) -> Result<(), AuthError> {
        if seconds_left <= 0 {
            return Ok(());
        }

        let rkey = self.refresh_key(hash_b64url);
        let ttl = (seconds_left + self.skew_secs).max(1);
        self.cache
            .insert_with_expiry(&rkey, &"rotated", std::time::Duration::from_secs(ttl as u64))
            .await?;

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

        let skey = self.session_key(session_id);
        let ttl = (session_ttl_secs + self.skew_secs).max(1);
        self.cache
            .insert_with_expiry(&skey, &"revoked", std::time::Duration::from_secs(ttl as u64))
            .await?;

        Ok(())
    }
}

impl From<CacheError> for AuthError {
    fn from(e: CacheError) -> Self {
        Self::Storage(format!("cache: {e}"))
    }
}
