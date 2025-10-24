use crate::auth_core::{
    errors::Result,
    models::{AccessClaims, RefreshPair, SignedToken, Uid},
};

pub trait PasswordHasher: Send + Sync {
    fn hash(&self, plain: &str) -> Result<String>;
    fn verify(&self, hash: &str, plain: &str) -> Result<bool>;
}

pub trait AccessTokenIssuer: Send + Sync {
    fn issue_token(&self, user_id: Uid, session_id: Uid, roles: &[String]) -> Result<SignedToken>;
    fn validate(&self, token: &str) -> Result<AccessClaims>;
}

pub trait RefreshTokenFactory: Send + Sync {
    fn new_pair(&self) -> RefreshPair;
    fn hash(&self, token_plain: &str) -> Vec<u8>;
}
