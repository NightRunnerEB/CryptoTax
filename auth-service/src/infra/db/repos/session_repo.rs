// use async_trait::async_trait;
// use chrono::{DateTime, Utc};
// use sqlx::{Pool, Postgres, prelude::FromRow};

// use crate::auth_core::{errors::AuthError, models::*, ports::SessionRepo};

// #[derive(Clone)]
// pub struct SqlxSessionRepo {
//     pub pool: Pool<Postgres>,
// }

// #[async_trait]
// impl SessionRepo for SqlxSessionRepo {
//     async fn create(
//         &self,
//         user_id: Uid,
//         ip: Option<String>,
//         ua: Option<String>,
//     ) -> Result<Session, AuthError> {
//         let id = Uid::new_v4();
//         let r = sqlx::query_as!(
//             SessionRow,
//             r#"
//             INSERT INTO sessions (id, user_id, ip, user_agent)
//             VALUES ($1, $2, $3, $4)
//             RETURNING id, user_id, created_at, last_seen_at, ip, user_agent
//             "#,
//             id,
//             user_id,
//             ip,
//             ua
//         )
//         .fetch_one(&self.pool)
//         .await
//         .map_err(|e| AuthError::Storage(e.to_string()))?;

//         Ok(Session {
//             id: r.id,
//             user_id: r.user_id,
//             status: r.
//             created_at: r.created_at,
//             last_seen_at: r.last_seen_at,
//             ip: r.ip,
//             user_agent: r.user_agent,
//         })
//     }

//     async fn revoke_all_for_session(&self, session_id: Uid) -> Result<(), AuthError> {
//         sqlx::query!(
//             r#"UPDATE refresh_tokens SET revoked_at = now() WHERE session_id = $1 AND revoked_at IS NULL"#,
//             session_id
//         )
//         .execute(&self.pool).await
//         .map_err(|e| AuthError::Storage(e.to_string()))?;
//         Ok(())
//     }

//     async fn list_for_user(&self, user_id: Uid) -> Result<Vec<Session>, AuthError> {
//         let rows = sqlx::query_as!(
//             SessionRow,
//             r#"SELECT id, user_id, created_at, last_seen_at, ip, user_agent
//                FROM sessions WHERE user_id = $1 ORDER BY created_at DESC"#,
//             user_id
//         )
//         .fetch_all(&self.pool)
//         .await
//         .map_err(|e| AuthError::Storage(e.to_string()))?;

//         Ok(rows
//             .into_iter()
//             .map(|r| Session {
//                 id: r.id,
//                 user_id: r.user_id,
//                 created_at: r.created_at,
//                 last_seen_at: r.last_seen_at,
//                 ip: r.ip,
//                 user_agent: r.user_agent,
//             })
//             .collect())
//     }
// }
