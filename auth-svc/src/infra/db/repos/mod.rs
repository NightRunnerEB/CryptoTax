pub mod email_verif_repo;
pub mod refresh_repo;
pub mod session_repo;
pub mod user_repo;

pub use email_verif_repo::*;
pub use refresh_repo::*;
pub use session_repo::*;
pub use user_repo::*;

use chrono::{DateTime, Utc};
use sqlx::FromRow;

use crate::auth_core::{
    errors::AuthError,
    models::{RefreshToken, Session, SessionStatus, Uid, UserStatus, UserWithHash},
};

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
