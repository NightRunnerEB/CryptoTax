pub mod email_verif_repo;
pub mod refresh_repo;
pub mod session_repo;
pub mod user_repo;

use chrono::{DateTime, Utc};
pub use email_verif_repo::*;
pub use refresh_repo::*;
pub use session_repo::*;
use sqlx::FromRow;
pub use user_repo::*;

use crate::auth_core::{
    errors::AuthError,
    models::{RefreshToken, Session, SessionStatus, Uid, UserStatus, UserWithHash},
};

#[cfg(test)]
pub(crate) mod test_utils {
    use chrono::{Duration, Utc};
    use sqlx::{PgPool, Row};
    use uuid::Uuid;

    use crate::db::make_pool;

    pub async fn test_pool() -> PgPool {
        dotenvy::dotenv().ok();
        let url = std::env::var("AUTH_TEST_DATABASE_URL")
            .or_else(|_| std::env::var("DATABASE_URL"))
            .expect("AUTH_TEST_DATABASE_URL or DATABASE_URL must be set for integration tests");

        let pool = make_pool(&url, 5, 5).await.expect("connect test database");
        cleanup_db(&pool).await;
        pool
    }

    pub async fn cleanup_db(pool: &PgPool) {
        sqlx::query(
            r#"
            TRUNCATE TABLE
                email_verifications,
                refresh_tokens,
                sessions,
                users
            RESTART IDENTITY CASCADE
            "#,
        )
        .execute(pool)
        .await
        .expect("truncate tables");
    }

    pub async fn insert_user(pool: &PgPool, status: &str) -> Uuid {
        let email = format!("{}@example.com", Uuid::new_v4());
        let row = sqlx::query(
            r#"
            INSERT INTO users (email, password_hash, status)
            VALUES ($1::citext, $2, $3::user_status)
            RETURNING id
            "#,
        )
        .bind(email)
        .bind("hashed-password")
        .bind(status)
        .fetch_one(pool)
        .await
        .expect("insert user");

        row.get("id")
    }

    pub async fn insert_session(pool: &PgPool, user_id: Uuid) -> Uuid {
        let row = sqlx::query(
            r#"
            INSERT INTO sessions (user_id, status)
            VALUES ($1, 'Active'::session_status)
            RETURNING id
            "#,
        )
        .bind(user_id)
        .fetch_one(pool)
        .await
        .expect("insert session");

        row.get("id")
    }

    pub fn refresh_hash(seed: u8) -> Vec<u8> {
        vec![seed; 32]
    }

    pub fn refresh_expiry() -> chrono::DateTime<Utc> {
        Utc::now() + Duration::hours(2)
    }
}

#[derive(FromRow)]
pub struct UserRow {
    pub id: Uid,
    pub status: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub password_hash: String,
}

impl TryFrom<UserRow> for UserWithHash {
    type Error = AuthError;

    fn try_from(value: UserRow) -> Result<Self, Self::Error> {
        let status = match value.status.as_str() {
            "Active" => UserStatus::Active,
            "Pending" => UserStatus::Pending,
            "Blocked" => UserStatus::Blocked,
            other => return Err(AuthError::Storage(format!("invalid user_status: {other}"))),
        };
        Ok(UserWithHash {
            id: value.id,
            email: value.email,
            status,
            created_at: value.created_at,
            password_hash: value.password_hash,
        })
    }
}

#[derive(sqlx::Type, Clone, Copy)]
#[sqlx(type_name = "session_status", rename_all = "PascalCase")]
pub enum PgSessionStatus {
    Active,
    Revoked,
    Closed,
}

impl From<SessionStatus> for PgSessionStatus {
    fn from(s: SessionStatus) -> Self {
        match s {
            SessionStatus::Active => Self::Active,
            SessionStatus::Revoked => Self::Revoked,
            SessionStatus::Closed => Self::Closed,
        }
    }
}

#[derive(FromRow)]
struct SessionRow {
    id: Uid,
    user_id: Uid,
    status: String,
    created_at: DateTime<Utc>,
    last_seen_at: DateTime<Utc>,
    ip: Option<String>,
    user_agent: Option<String>,
}

impl TryFrom<SessionRow> for Session {
    type Error = AuthError;

    fn try_from(value: SessionRow) -> Result<Self, Self::Error> {
        let status = match value.status.as_str() {
            "Active" => SessionStatus::Active,
            "Revoked" => SessionStatus::Revoked,
            "Closed" => SessionStatus::Closed,
            other => return Err(AuthError::Storage(format!("invalid session_status: {other}"))),
        };
        Ok(Session {
            id: value.id,
            user_id: value.user_id,
            status,
            created_at: value.created_at,
            last_seen_at: value.last_seen_at,
            ip: value.ip,
            user_agent: value.user_agent,
        })
    }
}

#[derive(FromRow)]
pub struct RefreshRow {
    pub jti: Uid,
    pub user_id: Uid,
    pub session_id: Uid,
    pub expires_at: DateTime<Utc>,
    pub rotated_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
}

impl TryFrom<RefreshRow> for RefreshToken {
    type Error = AuthError;

    fn try_from(r: RefreshRow) -> Result<Self, Self::Error> {
        Ok(RefreshToken {
            jti: r.jti,
            user_id: r.user_id,
            session_id: r.session_id,
            expires_at: r.expires_at,
            rotated_at: r.rotated_at,
            revoked_at: r.revoked_at,
        })
    }
}
