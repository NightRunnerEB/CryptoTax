// use crate::auth_core::{errors::AuthError, models::Uid, ports::RevocationCache};
// use async_trait::async_trait;
// use cache::{Cache, CacheConfig, CacheError};
// use std::sync::Arc;

// #[derive(Clone)]
// pub struct CacheRevocation {
//     cache: Arc<Cache>,
//     ns: &'static str,
// }

// impl CacheRevocation {
//     pub async fn new(config: CacheConfig) -> Result<Self, AuthError> {
//         let cache = Cache::new(&config).await?;
//         Ok(Self { cache, ns: "auth" })
//     }

//     #[inline]
//     fn family_key(&self, family_id: Uid) -> String {
//         format!("{}:bl:family:{family_id}", self.ns)
//     }

//     #[inline]
//     fn refresh_key(&self, hash_b64url: &str) -> String {
//         format!("{}:bl:refresh:{hash_b64url}", self.ns)
//     }
// }

// #[async_trait]
// impl RevocationCache for CacheRevocation {
//     async fn is_refresh_blocked(
//         &self,
//         session_id: Uid,
//         hash_b64url: &str,
//     ) -> Result<bool, AuthError> {
//         let fam = self.family_key(session_id);
//         let refk = self.refresh_key(hash_b64url);

//         let flags = self.cache.exists_many(&[&fam, &refk]).await?;
//         Ok(flags.into_iter().any(|b| b))
//     }

//     async fn mark_refresh_rotated(
//         &self,
//         hash_b64url: &str,
//         seconds_left: i64,
//     ) -> Result<(), AuthError> {
//         if seconds_left <= 0 {
//             return Ok(());
//         }
//         let rkey = self.refresh_key(hash_b64url);
//         // сохраняем любую метку; значение нам не важно
//         self.cache
//             .insert_with_expiry(
//                 &rkey,
//                 &"rotated",
//                 std::time::Duration::from_secs(seconds_left as u64),
//             )
//             .await?;

//         Ok(())
//     }

//     async fn revoke_family(&self, family_id: Uid, family_ttl_secs: i64) -> Result<(), AuthError> {
//         if family_ttl_secs <= 0 {
//             return Ok(());
//         }
//         let fkey = self.family_key(family_id);
//         self.cache
//             .insert_with_expiry(
//                 &fkey,
//                 &"revoked",
//                 std::time::Duration::from_secs(family_ttl_secs as u64),
//             )
//             .await?;

//         Ok(())
//     }
// }

// impl From<CacheError> for AuthError {
//     fn from(e: CacheError) -> Self {
//         Self::Storage(format!("cache: {e}"))
//     }
// }
