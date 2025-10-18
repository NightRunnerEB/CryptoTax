use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::errors::AuthError;

pub type Uid = Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum UserStatus {
    Active,
    Pending,
    Blocked,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub id: Uid,
    pub email: String,
    pub status: UserStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct UserWithHash {
    pub id: Uid,
    pub email: String,
    pub status: UserStatus,
    pub created_at: DateTime<Utc>,
    pub password_hash: String,
}
impl UserWithHash {
    pub fn into_user(self) -> User {
        User {
            id: self.id,
            status: self.status,
            email: self.email,
            created_at: self.created_at,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SessionStatus {
    Active,
    Revoked,
    Closed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Session {
    pub id: Uid,
    pub user_id: Uid,
    pub status: SessionStatus,
    pub created_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub ip: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Clone, Debug)]
pub struct RefreshToken {
    pub jti: Uid,
    pub user_id: Uid,
    pub session_id: Uid,
    pub expires_at: DateTime<Utc>,
    pub rotated_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
}

impl RefreshToken {
    pub fn ensure_active(&self) -> Result<(), AuthError> {
        use super::errors::AuthError::*;
        let now = Utc::now();
        if self.revoked_at.is_some() {
            return Err(TokenInvalid);
        }
        if self.expires_at <= now {
            return Err(TokenExpired);
        }
        if self.rotated_at.is_some() {
            return Err(TokenReuse);
        }
        Ok(())
    }
    pub fn seconds_left(&self) -> i64 {
        (self.expires_at - Utc::now()).num_seconds().max(0)
    }
}

#[derive(Clone, Debug)]
pub struct NewRefresh {
    pub jti: Uid,
    pub user_id: Uid,
    pub session_id: Uid,
    pub token_hash: Vec<u8>,
    pub parent_jti: Option<Uid>,
    pub expires_at: DateTime<Utc>,
}
impl NewRefresh {
    pub fn from_pair(p: &RefreshPair, user_id: Uid, session_id: Uid) -> Self {
        Self {
            jti: p.jti,
            user_id,
            session_id,
            token_hash: p.token_hash.clone(),
            parent_jti: None,
            expires_at: p.expires_at,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SignedToken {
    pub token: String,
    pub exp: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccessClaims {
    pub sub: String,
    pub jti: String,
    pub iss: String,
    pub aud: String,
    pub sid: String,
    pub iat: i64,
    pub exp: i64,
    pub roles: Vec<String>,
}

#[derive(Clone)]
pub struct RefreshPair {
    pub token_plain: String,
    pub token_hash: Vec<u8>,
    pub jti: Uid,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tokens {
    pub access_token: String,
    pub refresh_token: String,
    pub access_expires_in: i64,
    pub refresh_expires_in: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct LoginResult {
    pub user: User,
    pub session: Session,
    pub tokens: Tokens,
}

#[derive(Clone, Debug)]
pub struct EmailVerificationRec {
    pub id: Uuid,
    pub user_id: Uid,
    pub token_hash: Vec<u8>,
    pub sent_to: String,
    pub expires_at: DateTime<Utc>,
    pub consumed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefreshBlockReason {
    Rotated,
    RevokedSession,
}
