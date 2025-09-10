//! # Redis Cache Driver
//!
//! This module implements a cache driver using Redis.
use std::time::Duration;

use async_trait::async_trait;
use bb8::Pool;
use bb8_redis::{
    RedisConnectionManager, bb8,
    redis::{AsyncCommands, cmd, pipe},
};

use crate::{CacheError, CacheResult, RedisCacheConfig};

use super::CacheDriver;

/// Represents the Redis cache driver.
#[derive(Clone, Debug)]
pub struct Redis {
    pool: Pool<RedisConnectionManager>,
}

impl Redis {
    #[must_use]
    pub async fn new(config: &RedisCacheConfig) -> CacheResult<Box<dyn CacheDriver>> {
        let manager = RedisConnectionManager::new(config.uri.clone())?;
        let pool = Pool::builder().max_size(config.max_size).build(manager).await?;

        Ok(Box::new(Self { pool }))
    }
}

#[async_trait]
impl CacheDriver for Redis {
    async fn ping(&self) -> CacheResult<()> {
        let mut conn = self.pool.get().await?;
        match conn.ping::<Option<String>>().await? {
            Some(_) => Ok(()),
            None => Err(CacheError::Any("Redis ping failed".into())),
        }
    }

    async fn contains_key(&self, key: &str) -> CacheResult<bool> {
        let mut connection = self.pool.get().await?;
        Ok(connection.exists(key).await?)
    }

    async fn get(&self, key: &str) -> CacheResult<Option<String>> {
        let mut conn = self.pool.get().await?;
        let result: Option<String> = conn.get(key).await?;
        Ok(result)
    }

    async fn insert(&self, key: &str, value: &str) -> CacheResult<()> {
        let mut conn = self.pool.get().await?;
        conn.set::<_, _, ()>(key, value).await?;
        Ok(())
    }

    async fn insert_with_expiry(
        &self,
        key: &str,
        value: &str,
        duration: Duration,
    ) -> CacheResult<()> {
        let mut conn = self.pool.get().await?;
        conn.set_ex::<_, _, ()>(key, value, duration.as_secs()).await?;
        Ok(())
    }

    async fn remove(&self, key: &str) -> CacheResult<()> {
        let mut conn = self.pool.get().await?;
        conn.del::<_, ()>(key).await?;
        Ok(())
    }

    async fn clear(&self) -> CacheResult<()> {
        let mut conn = self.pool.get().await?;
        cmd("FLUSHDB").query_async::<()>(&mut *conn).await?;
        Ok(())
    }

    async fn exists_many(&self, keys: &[&str]) -> CacheResult<Vec<bool>> {
        let mut conn = self.pool.get().await?;
        let mut p = pipe();
        for k in keys {
            p.cmd("EXISTS").arg(k);
        }
        let ints: Vec<i64> = p.query_async(&mut *conn).await?;
        Ok(ints.into_iter().map(|n| n > 0).collect())
    }
}
