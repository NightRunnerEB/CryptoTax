use async_trait::async_trait;
use chrono::{DateTime, Utc};

use super::{errors::AuthError, models::*};

#[async_trait]
pub trait UserRepo: Send + Sync {
    async fn create_user(&self, email: &str, password_hash: &str) -> Result<Option<Uid>, AuthError>;
    async fn find_by_email(&self, email_lower: &str) -> Result<Option<UserWithHash>, AuthError>;
    async fn activate(&self, user_id: Uid) -> Result<bool, AuthError>;
    async fn update_password(&self, user_id: Uid, password_hash: &str) -> Result<(), AuthError>;
}

#[async_trait]
pub trait EmailVerificationRepo: Send + Sync {
    async fn create_token(&self, user_id: Uid, token_hash: Vec<u8>, sent_to: &str, expires_at: DateTime<Utc>) -> Result<(), AuthError>;
    async fn revoke_all_for_user(&self, user_id: Uid) -> Result<(), AuthError>;
    async fn consume_by_hash(&self, token_hash: &[u8]) -> Result<Option<Uid>, AuthError>;
}

#[async_trait]
pub trait Mailer: Send + Sync {
    async fn send_verification(&self, to: &str, verify_link: &str) -> Result<(), AuthError>;
}

#[async_trait]
pub trait SessionRepo: Send + Sync {
    async fn create(&self, user_id: Uid, ip: Option<String>, ua: Option<String>) -> Result<Session, AuthError>;
    async fn get(&self, session_id: Uid) -> Result<Option<Session>, AuthError>;
    async fn set_status(&self, session_id: Uid, status: SessionStatus) -> Result<(), AuthError>;
    async fn touch(&self, session_id: Uid) -> Result<(), AuthError>;
    async fn list_for_user(&self, user_id: Uid) -> Result<Vec<Session>, AuthError>;
}

#[async_trait]
pub trait RefreshRepo: Send + Sync {
    async fn get_by_hash(&self, hash: &[u8]) -> Result<Option<RefreshToken>, AuthError>;
    async fn mark_rotated(&self, jti: Uid) -> Result<bool, AuthError>;
    async fn insert(&self, rec: NewRefresh) -> Result<(), AuthError>;
    async fn revoke_all_for_session(&self, session_id: Uid) -> Result<(), AuthError>;
}

#[async_trait]
pub trait RevocationCache: Send + Sync {
    async fn check_refresh(&self, session_id: Uid, token_hash_b64: &str) -> Result<Option<RefreshBlockReason>, AuthError>;
    async fn mark_refresh_rotated(&self, token_hash_b64: &str, seconds_left: i64) -> Result<(), AuthError>;
    async fn revoke_all_for_session(&self, session_id: Uid, session_ttl_secs: i64) -> Result<(), AuthError>;
}
