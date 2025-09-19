//! # Null Cache Driver
//!
//! The Null Cache Driver is the default cache driver implemented to simplify
//! the user workflow by avoiding the need for feature flags or optional cache
//! driver configurations in Dev/Test stage.
use std::time::Duration;

use async_trait::async_trait;

use super::CacheDriver;
use crate::{CacheError, CacheResult};

#[derive(Debug)]
pub struct Null {}

impl Null {
    #[must_use]
    pub fn new() -> Box<dyn CacheDriver> {
        Box::new(Null {})
    }
}

#[async_trait]
impl CacheDriver for Null {
    async fn ping(&self) -> CacheResult<()> {
        Err(CacheError::Any("Operation not supported by null cache".into()))
    }

    async fn contains_key(&self, _key: &str) -> CacheResult<bool> {
        Err(CacheError::Any("Operation not supported by null cache".into()))
    }

    async fn get(&self, _key: &str) -> CacheResult<Option<String>> {
        Ok(None)
    }

    async fn get_many(&self, _key: &[&str]) -> CacheResult<Vec<Option<String>>> {
        Ok(vec![None])
    }

    async fn insert(&self, _key: &str, _value: &str) -> CacheResult<()> {
        Err(CacheError::Any("Operation not supported by null cache".into()))
    }

    async fn insert_with_expiry(
        &self,
        _key: &str,
        _value: &str,
        _duration: Duration,
    ) -> CacheResult<()> {
        Err(CacheError::Any("Operation not supported by null cache".into()))
    }

    async fn remove(&self, _key: &str) -> CacheResult<()> {
        Err(CacheError::Any("Operation not supported by null cache".into()))
    }

    async fn clear(&self) -> CacheResult<()> {
        Err(CacheError::Any("Operation not supported by null cache".into()))
    }

    async fn exists_many(&self, _keys: &[&str]) -> CacheResult<Vec<bool>> {
        Err(CacheError::Any("Operation not supported by null cache".into()))
    }
}
