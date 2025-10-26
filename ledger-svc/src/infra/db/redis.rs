// use std::sync::Arc;

// use cache::{Cache, CacheConfig, CacheError, RedisCacheConfig};

// use crate::infra::config::RedisConfig;

// #[derive(Clone)]
// pub struct RedisCache {
//     cache: Arc<Cache>,
//     skew_secs: i64,
//     ns: &'static str,
// }

// impl RedisCache {
//     pub async fn new(cfg: RedisConfig) -> Result<Self, CacheError> {
//         let cache_config = CacheConfig::Redis(RedisCacheConfig {
//             url: cfg.url,
//             max_size: cfg.max_size,
//         });
//         let cache = Cache::new(&cache_config).await?;
//         Ok(Self {
//             cache,
//             skew_secs: cfg.skew_secs,
//             ns: "auth",
//         })
//     }
// }

// #[async_trait]
// impl RevocationCache for RedisCache {
//     async fn check_refresh(
//         &self,
//         session_id: Uid,
//         token_hash_b64: &str,
//     ) -> Result<Option<RefreshBlockReason>, CacheError> {
//         let k_refresh = self.key_refresh(token_hash_b64);
//         let k_session = self.key_session(session_id);

//         let vals: Vec<Option<String>> = self.cache.get_many(&[&k_refresh, &k_session]).await?;
//         if let Some(v) = vals.get(0).and_then(|x| x.as_ref()) {
//             if v == "rotated" {
//                 return Ok(Some(RefreshBlockReason::Rotated));
//             }
//         }
//         if let Some(v) = vals.get(1).and_then(|x| x.as_ref()) {
//             if v == "revoked" {
//                 return Ok(Some(RefreshBlockReason::RevokedSession));
//             }
//         }
//         Ok(None)
//     }
// }
