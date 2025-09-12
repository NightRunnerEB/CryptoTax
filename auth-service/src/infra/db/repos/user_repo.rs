use async_trait::async_trait;
use sqlx::PgPool;

use crate::{
    auth_core::{
        errors::AuthError,
        models::{Uid, UserWithHash},
        ports::UserRepo,
    },
    infra::repos::UserRow,
};

pub struct PgUserRepo {
    pool: PgPool,
}

impl PgUserRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepo for PgUserRepo {
    async fn create_user(
        &self,
        email: &str,
        password_hash: &str,
    ) -> Result<Option<Uid>, AuthError> {
        let rec = sqlx::query!(
            r#"
            INSERT INTO users (email, password_hash)
            VALUES ($1::citext, $2)
            ON CONFLICT (email) DO NOTHING
            RETURNING id
            "#,
            email,
            password_hash
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AuthError::Storage(format!("users.insert: {e}")))?;

        Ok(rec.map(|r| r.id))
    }

    async fn find_by_email(&self, email_norm: &str) -> Result<Option<UserWithHash>, AuthError> {
        let row = sqlx::query_as!(
            UserRow,
            r#"
            SELECT id, email, status::text AS "status!", created_at, password_hash
            FROM users
            WHERE email = $1::citext
            "#,
            email_norm
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AuthError::Storage(format!("users.find_by_email: {e}")))?;

        Ok(match row {
            None => None,
            Some(r) => Some(r.try_into()?),
        })
    }
}
