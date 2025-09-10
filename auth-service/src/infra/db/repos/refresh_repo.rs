// use async_trait::async_trait;
// use chrono::{DateTime, Utc};
// use sqlx::{Pool, Postgres, prelude::FromRow};

// use crate::auth_core::{errors::AuthError, models::*, ports::RefreshRepo};

// #[derive(Clone)]
// pub struct SqlxRefreshRepo { pub pool: Pool<Postgres> }

// #[async_trait]
// impl RefreshRepo for SqlxRefreshRepo {
//     async fn get_by_hash(&self, hash: &[u8]) -> Result<Option<RefreshRecord>, AuthError> {
//         let r = sqlx::query_as!(
//             RefreshRow,
//             r#"
//             SELECT jti, user_id, session_id, family_id, expires_at, rotated_at, revoked_at
//             FROM refresh_tokens
//             WHERE token_hash = $1
//             "#,
//             hash
//         )
//         .fetch_optional(&self.pool)
//         .await
//         .map_err(|e| AuthError::Storage(e.to_string()))?;

//         Ok(r.map(|x| RefreshRecord {
//             jti: x.jti, user_id: x.user_id, session_id: x.session_id, family_id: x.family_id,
//             expires_at: x.expires_at, rotated_at: x.rotated_at, revoked_at: x.revoked_at
//         }))
//     }

//     async fn mark_rotated(&self, jti: Uid) -> Result<(), AuthError> {
//         sqlx::query!(r#"UPDATE refresh_tokens SET rotated_at = now() WHERE jti = $1"#, jti)
//             .execute(&self.pool).await
//             .map_err(|e| AuthError::Storage(e.to_string()))?;
//         Ok(())
//     }

//     async fn insert(&self, rec: NewRefresh) -> Result<(), AuthError> {
//         sqlx::query!(
//             r#"
//             INSERT INTO refresh_tokens
//               (jti, user_id, session_id, token_hash, family_id, parent_jti, expires_at)
//             VALUES ($1, $2, $3, $4, $5, $6, $7)
//             "#,
//             rec.jti, rec.user_id, rec.session_id, rec.token_hash, rec.family_id, rec.parent_jti, rec.expires_at
//         )
//         .execute(&self.pool).await
//         .map_err(|e| AuthError::Storage(e.to_string()))?;
//         Ok(())
//     }

//     async fn revoke_family(&self, family_id: Uid) -> Result<(), AuthError> {
//         sqlx::query!(
//             r#"UPDATE refresh_tokens SET revoked_at = now() WHERE family_id = $1 AND revoked_at IS NULL"#,
//             family_id
//         )
//         .execute(&self.pool).await
//         .map_err(|e| AuthError::Storage(e.to_string()))?;
//         Ok(())
//     }
// }
