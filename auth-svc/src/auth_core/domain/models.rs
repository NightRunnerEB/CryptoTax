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
pub struct RegisterTaxProfile {
    pub inn: String,
    pub last_name: String,
    pub first_name: String,
    #[serde(default)]
    pub middle_name: String,
    pub jurisdiction: String,
    pub timezone: String,
    #[serde(default)]
    pub phone: String,
    #[serde(default)]
    pub wallets: Vec<String>,
    pub tax_residency_status: String,
    pub taxpayer_type: String,
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

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};
    use uuid::Uuid;

    use super::RefreshToken;
    use crate::auth_core::errors::AuthError;

    fn make_refresh() -> RefreshToken {
        RefreshToken {
            jti: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            session_id: Uuid::new_v4(),
            expires_at: Utc::now() + Duration::minutes(30),
            rotated_at: None,
            revoked_at: None,
        }
    }

    #[test]
    fn ensure_active_returns_ok_for_valid_token() {
        let token = make_refresh();
        assert!(token.ensure_active().is_ok());
    }

    #[test]
    fn ensure_active_returns_token_invalid_for_revoked_token() {
        let mut token = make_refresh();
        token.revoked_at = Some(Utc::now());

        let err = token.ensure_active().expect_err("revoked token should fail");
        assert!(matches!(err, AuthError::TokenInvalid));
    }

    #[test]
    fn ensure_active_returns_token_expired_for_expired_token() {
        let mut token = make_refresh();
        token.expires_at = Utc::now() - Duration::seconds(1);

        let err = token.ensure_active().expect_err("expired token should fail");
        assert!(matches!(err, AuthError::TokenExpired));
    }

    #[test]
    fn ensure_active_returns_token_reuse_for_rotated_token() {
        let mut token = make_refresh();
        token.rotated_at = Some(Utc::now());

        let err = token.ensure_active().expect_err("rotated token should fail");
        assert!(matches!(err, AuthError::TokenReuse));
    }

    #[test]
    fn seconds_left_never_negative() {
        let mut token = make_refresh();
        token.expires_at = Utc::now() - Duration::hours(1);
        assert_eq!(token.seconds_left(), 0);
    }
}
