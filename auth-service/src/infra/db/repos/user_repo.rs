// use async_trait::async_trait;
// use sqlx::{Pool, Postgres};
// use uuid::Uuid;

// use crate::auth_core::{errors::AuthError, models::*, ports::UserRepo};
// use super::UserRow;

// #[derive(Clone)]
// pub struct PgUserRepo {
//     pub pool: Pool<Postgres>,
// }

// #[async_trait]
// impl UserRepo for PgUserRepo {
//     async fn create(&self, email: &str, password_hash: &str) -> Result<User, AuthError> {
//         let id = Uuid::new_v4();
//         let rec = sqlx::query_as!(
//             UserRow,
//             r#"
//             INSERT INTO users (id, email, password_hash)
//             VALUES ($1, $2, $3)
//             RETURNING id, status, email, created_at, password_hash
//             "#,
//             id,
//             email,
//             password_hash
//         )
//         .fetch_one(&self.pool)
//         .await
//         .map_err(|e| {
//             if let Some(db_err) = e.as_database_error() {
//                 if db_err.code().map(|c| c == "23505").unwrap_or(false) {
//                     return AuthError::EmailExists;
//                 }
//             }
//             AuthError::Storage(e.to_string())
//         })?;

//         Ok(User {
//             id: rec.id,
//             status: rec.status,
//             email: rec.email,
//             created_at: rec.created_at,
//         })
//     }

//     async fn find_by_email(&self, email_lower: &str) -> Result<Option<UserWithHash>, AuthError> {
//         let rec = sqlx::query_as!(
//             UserRow,
//             r#"
//             SELECT id, user_status, email, created_at, password_hash
//             FROM users
//             WHERE lower(email) = $1
//             "#,
//             email_lower
//         )
//         .fetch_optional(&self.pool)
//         .await
//         .map_err(|e| AuthError::Storage(e.to_string()))?;

//         Ok(rec.map(|r| UserWithHash {
//             id: r.id,
//             status: r.status,
//             email: r.email,
//             created_at: r.created_at,
//             password_hash: r.password_hash,
//         }))
//     }
// }
