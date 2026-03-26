use axum::async_trait;
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
        Self {
            pool,
        }
    }
}

#[async_trait]
impl UserRepo for PgUserRepo {
    async fn create_user(&self, email: &str, password_hash: &str) -> Result<Option<Uid>, AuthError> {
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

    async fn activate(&self, user_id: Uid) -> Result<bool, AuthError> {
        let res = sqlx::query!(
            r#"
            UPDATE users
            SET status = 'Active'::user_status
            WHERE id = $1
              AND status = 'Pending'::user_status
            "#,
            user_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::Storage(format!("users.activate: {e}")))?;
        Ok(res.rows_affected() == 1)
    }

    async fn update_password(&self, user_id: Uid, new_hash: &str) -> Result<(), AuthError> {
        let rec = sqlx::query_scalar!(
            r#"
            UPDATE users
            SET password_hash = $2
            WHERE id = $1
            AND status IN ('Pending'::user_status, 'Active'::user_status)
            RETURNING 1
            "#,
            user_id,
            new_hash
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AuthError::Storage(format!("users.update_password: {e}")))?;

        match rec {
            Some(_) => Ok(()),
            None => Err(AuthError::PasswordUpdateNotAllowed),
        }
    }

    async fn delete_pending_user(&self, user_id: Uid) -> Result<bool, AuthError> {
        let res = sqlx::query(
            r#"
            DELETE FROM users
            WHERE id = $1
              AND status = 'Pending'::user_status
            "#,
        )
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::Storage(format!("users.delete_pending_user: {e}")))?;

        Ok(res.rows_affected() == 1)
    }
}

#[cfg(test)]
mod tests {
    use serial_test::serial;
    use uuid::Uuid;

    use super::*;
    use crate::auth_core::{errors::AuthError, models::UserStatus};
    use crate::infra::db::repos::test_utils;

    #[tokio::test]
    #[ignore = "manual integration"]
    #[serial]
    async fn create_and_find_user() {
        let pool = test_utils::test_pool().await;
        let repo = PgUserRepo::new(pool.clone());

        let email = format!("{}@example.com", Uuid::new_v4());
        let user_id =
            repo.create_user(&email, "hash-1").await.expect("create user should succeed").expect("id should be returned");

        let duplicate = repo.create_user(&email, "hash-2").await.expect("duplicate call should not fail");
        assert!(duplicate.is_none(), "duplicate email should not create new user");

        let found = repo.find_by_email(&email).await.expect("find should succeed").expect("user must exist");
        assert_eq!(found.id, user_id);
        assert_eq!(found.email.to_lowercase(), email.to_lowercase());
        assert!(matches!(found.status, UserStatus::Pending));
        assert_eq!(found.password_hash, "hash-1");

        test_utils::cleanup_db(&pool).await;
    }

    #[tokio::test]
    #[ignore = "manual integration"]
    #[serial]
    async fn activate_and_delete_pending() {
        let pool = test_utils::test_pool().await;
        let repo = PgUserRepo::new(pool.clone());

        let email = format!("{}@example.com", Uuid::new_v4());
        let user_id = repo.create_user(&email, "hash").await.expect("create user").expect("user id");

        let activated = repo.activate(user_id).await.expect("activate should work");
        assert!(activated);

        let activated_again = repo.activate(user_id).await.expect("second activate should work");
        assert!(!activated_again);

        let deleted_pending = repo.delete_pending_user(user_id).await.expect("delete pending should work");
        assert!(!deleted_pending, "active user should not be deleted as pending");

        test_utils::cleanup_db(&pool).await;
    }

    #[tokio::test]
    #[ignore = "manual integration"]
    #[serial]
    async fn update_password_for_blocked_user_is_rejected() {
        let pool = test_utils::test_pool().await;
        let repo = PgUserRepo::new(pool.clone());

        let user_id = test_utils::insert_user(&pool, "Blocked").await;

        let err = repo.update_password(user_id, "new-hash").await.expect_err("blocked user must be rejected");
        assert!(matches!(err, AuthError::PasswordUpdateNotAllowed));

        test_utils::cleanup_db(&pool).await;
    }
}
