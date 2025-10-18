use crate::auth_core::{
    errors::AuthError,
    models::{AccessClaims, RefreshPair, SignedToken, Uid},
};

pub trait PasswordHasher {
    fn hash(&self, plain: &str) -> Result<String, AuthError>;
    fn verify(&self, hash: &str, plain: &str) -> Result<bool, AuthError>;
}

pub trait AccessTokenIssuer {
    fn issue_token(
        &self,
        user_id: Uid,
        session_id: Uid,
        roles: &[String],
    ) -> Result<SignedToken, AuthError>;
    fn validate(&self, token: &str) -> Result<AccessClaims, AuthError>;
}

pub trait RefreshTokenFactory {
    fn new_pair(&self) -> RefreshPair;
    fn hash(&self, token_plain: &str) -> Vec<u8>;
}
