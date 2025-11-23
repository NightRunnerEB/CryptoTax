use axum::async_trait;
use chrono::{DateTime, Utc};

use super::models::*;
use crate::auth_core::errors::Result;

#[async_trait]
pub trait UserRepo: Send + Sync {
    async fn create_user(&self, email: &str, password_hash: &str) -> Result<Option<Uid>>;
    async fn find_by_email(&self, email_lower: &str) -> Result<Option<UserWithHash>>;
    async fn activate(&self, user_id: Uid) -> Result<bool>;
    async fn update_password(&self, user_id: Uid, password_hash: &str) -> Result<()>;
}

#[async_trait]
pub trait EmailVerificationRepo: Send + Sync {
    async fn create_token(&self, user_id: Uid, token_hash: Vec<u8>, sent_to: &str, expires_at: DateTime<Utc>) -> Result<()>;
    async fn revoke_all_for_user(&self, user_id: Uid) -> Result<()>;
    async fn consume_by_hash(&self, token_hash: &[u8]) -> Result<Option<Uid>>;
}

#[async_trait]
pub trait Mailer: Send + Sync {
    async fn send_verification(&self, to: &str, verify_link: &str) -> Result<()>;
}

#[async_trait]
pub trait SessionRepo: Send + Sync {
    async fn create(&self, user_id: Uid, ip: Option<String>, ua: Option<String>) -> Result<Session>;
    async fn get(&self, session_id: Uid) -> Result<Option<Session>>;
    async fn set_status(&self, session_id: Uid, status: SessionStatus) -> Result<()>;
    async fn touch(&self, session_id: Uid) -> Result<()>;
    async fn list_for_user(&self, user_id: Uid) -> Result<Vec<Session>>;
}

#[async_trait]
pub trait RefreshRepo: Send + Sync {
    async fn get_by_hash(&self, hash: &[u8]) -> Result<Option<RefreshToken>>;
    async fn mark_rotated(&self, jti: Uid) -> Result<bool>;
    async fn insert(&self, rec: NewRefresh) -> Result<()>;
    async fn revoke_all_for_session(&self, session_id: Uid) -> Result<()>;
}

#[async_trait]
pub trait RevocationCache: Send + Sync {
    async fn check_refresh(&self, session_id: Uid, token_hash_b64: &str) -> Result<Option<RefreshBlockReason>>;
    async fn mark_refresh_rotated(&self, token_hash_b64: &str, seconds_left: i64) -> Result<()>;
    async fn revoke_all_for_session(&self, session_id: Uid, session_ttl_secs: i64) -> Result<()>;
}
