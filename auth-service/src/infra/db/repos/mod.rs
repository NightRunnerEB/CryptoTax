pub mod refresh_repo;
pub mod session_repo;
pub mod user_repo;

pub use refresh_repo::*;
pub use session_repo::*;
pub use user_repo::*;

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use crate::auth_core::models::{SessionStatus, UserStatus};

#[derive(FromRow)]
pub struct UserRow {
    pub id: Uuid,
    pub status: UserStatus,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub password_hash: String,
}

#[derive(FromRow)]
struct SessionRow {
    id: Uuid,
    user_id: Uuid,
    status: SessionStatus,
    created_at: DateTime<Utc>,
    last_seen_at: DateTime<Utc>,
    ip: Option<String>,
    user_agent: Option<String>,
}

#[derive(FromRow)]
struct RefreshRow {
  jti: Uuid,
  user_id: Uuid,
  session_id: Uuid,
  family_id: Uuid,
  expires_at: DateTime<Utc>,
  rotated_at: Option<DateTime<Utc>>,
  revoked_at: Option<DateTime<Utc>>,
}
