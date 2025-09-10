//! # Cache Drivers Module
//!
//! This module defines traits and implementations for cache drivers.
pub mod null;
#[cfg(feature = "cache_redis")]
pub mod redis;

use async_trait::async_trait;
use std::time::Duration;

use crate::CacheResult;

#[async_trait]
pub trait CacheDriver: Sync + Send {
    async fn ping(&self) -> CacheResult<()>;

    async fn contains_key(&self, key: &str) -> CacheResult<bool>;

    async fn get(&self, key: &str) -> CacheResult<Option<String>>;

    async fn insert(&self, key: &str, value: &str) -> CacheResult<()>;

    async fn insert_with_expiry(
        &self,
        key: &str,
        value: &str,
        duration: Duration,
    ) -> CacheResult<()>;

    async fn remove(&self, key: &str) -> CacheResult<()>;

    async fn clear(&self) -> CacheResult<()>;

    async fn exists_many(&self, keys: &[&str]) -> CacheResult<Vec<bool>>;
}
