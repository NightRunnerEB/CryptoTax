use serde::Serialize;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, AuthError>;

#[derive(Debug, Error, Serialize)]
pub enum AuthError {
    #[error("email is invalid")]
    EmailInvalid,
    #[error("password is too weak: {0}")]
    PasswordWeak(String),
    #[error("password cannot be updated for users in status Blocked")]
    PasswordUpdateNotAllowed,
    #[error("email already registered")]
    EmailAlreadyRegistered,
    #[error("email send failed")]
    EmailSendFailed,
    #[error("user not verified")]
    UserNotVerified,
    #[error("user has blocked")]
    UserBlocked,
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("token expired")]
    TokenExpired,
    #[error("token reuse detected")]
    TokenReuse,
    #[error("token invalid")]
    TokenInvalid,
    #[error("storage error: {0}")]
    Storage(String),
    #[error("internal error")]
    Internal,
}

impl From<anyhow::Error> for AuthError {
    fn from(_: anyhow::Error) -> Self {
        Self::Internal
    }
}
