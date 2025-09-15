//! # Cache Drivers Module
//!
//! This module defines traits and implementations for cache drivers.
pub mod drivers;

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{sync::Arc, time::Duration};

use crate::drivers::{CacheDriver, null::Null, redis::Redis};

pub type CacheResult<T> = std::result::Result<T, CacheError>;

/// Errors related to cache operations
#[derive(thiserror::Error, Debug)]
#[allow(clippy::module_name_repetitions)]
pub enum CacheError {
    #[error(transparent)]
    Any(#[from] Box<dyn std::error::Error + Send + Sync>),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),

    #[cfg(feature = "cache_redis")]
    #[error(transparent)]
    Redis(#[from] bb8_redis::redis::RedisError),

    #[cfg(feature = "cache_redis")]
    #[error(transparent)]
    RedisConnectionError(#[from] bb8_redis::bb8::RunError<bb8_redis::redis::RedisError>),
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(tag = "kind")]
pub enum CacheConfig {
    #[cfg(feature = "cache_redis")]
    Redis(RedisCacheConfig),
    #[default]
    Null,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RedisCacheConfig {
    pub url: String,
    pub max_size: u32,
}

pub struct Cache {
    pub driver: Box<dyn CacheDriver>,
}

impl Cache {
    #[must_use]
    pub async fn new(config: &CacheConfig) -> Result<Arc<Cache>, CacheError> {
        match &config {
            #[cfg(feature = "cache_redis")]
            CacheConfig::Redis(config) => {
                let driver = Redis::new(config).await?;
                Ok(Arc::new(Self { driver }))
            }
            CacheConfig::Null => {
                let driver = Null::new();
                Ok(Arc::new(Self { driver }))
            }
        }
    }

    pub async fn ping(&self) -> CacheResult<()> {
        self.driver.ping().await
    }

    pub async fn contains_key(&self, key: &str) -> CacheResult<bool> {
        self.driver.contains_key(key).await
    }

    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> CacheResult<Option<T>> {
        let result = self.driver.get(key).await?;
        if let Some(value) = result {
            let deserialized = serde_json::from_str::<T>(&value)
                .map_err(|e| CacheError::Deserialization(e.to_string()))?;
            Ok(Some(deserialized))
        } else {
            Ok(None)
        }
    }

    pub async fn insert<T: Serialize + Sync + ?Sized>(
        &self,
        key: &str,
        value: &T,
    ) -> CacheResult<()> {
        let serialized =
            serde_json::to_string(value).map_err(|e| CacheError::Serialization(e.to_string()))?;
        self.driver.insert(key, &serialized).await
    }

    pub async fn insert_with_expiry<T: Serialize + Sync + ?Sized>(
        &self,
        key: &str,
        value: &T,
        duration: Duration,
    ) -> CacheResult<()> {
        let serialized =
            serde_json::to_string(value).map_err(|e| CacheError::Serialization(e.to_string()))?;
        self.driver.insert_with_expiry(key, &serialized, duration).await
    }

    pub async fn remove(&self, key: &str) -> CacheResult<()> {
        self.driver.remove(key).await
    }

    pub async fn clear(&self) -> CacheResult<()> {
        self.driver.clear().await
    }

    pub async fn exists_many(&self, keys: &[&str]) -> CacheResult<Vec<bool>> {
        self.driver.exists_many(keys).await
    }
}
