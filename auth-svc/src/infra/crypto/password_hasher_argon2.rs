use argon2::{
    Algorithm, Argon2, Params, Version,
    password_hash::{PasswordHash, PasswordHasher as _, PasswordVerifier as _, SaltString},
};
use hmac::{Hmac, Mac};
use rand::rngs::OsRng;
use sha2::Sha256;

use crate::{
    auth_core::{errors::AuthError, services::PasswordHasher},
    infra::PepperSet,
};

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone, Copy)]
pub struct KdfParams {
    pub m_cost_kib: u32,
    pub t_cost: u32,
    pub p_lanes: u32,
}

pub struct Argon2Hasher {
    a2: Argon2<'static>,
    peppers: PepperSet,
}

impl Argon2Hasher {
    pub fn new(cfg: KdfParams, peppers: PepperSet) -> Result<Self, AuthError> {
        let params = Params::new(cfg.m_cost_kib, cfg.t_cost, cfg.p_lanes, None)
            .map_err(|_| AuthError::Internal)?;
        let a2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
        Ok(Self { a2, peppers })
    }

    /// HMAC(pepper, password || salt)
    fn prehash(
        &self,
        pepper: &[u8],
        password: &str,
        salt_bytes: &[u8],
    ) -> Result<Vec<u8>, AuthError> {
        let mut mac = HmacSha256::new_from_slice(pepper).map_err(|_| AuthError::Internal)?;
        mac.update(password.as_bytes());
        mac.update(salt_bytes);
        Ok(mac.finalize().into_bytes().to_vec())
    }
}

impl PasswordHasher for Argon2Hasher {
    fn hash(&self, plain: &str) -> Result<String, AuthError> {
        let salt = SaltString::generate(&mut OsRng);
        let pre = self.prehash(self.peppers.current(), plain, salt.as_str().as_bytes())?;
        let phc = self.a2.hash_password(&pre, &salt).map_err(|_| AuthError::Internal)?.to_string();
        Ok(phc)
    }

    fn verify(&self, hash: &str, plain: &str) -> Result<bool, AuthError> {
        let parsed = PasswordHash::new(hash).map_err(|_| AuthError::Internal)?;
        let salt = parsed.salt.ok_or(AuthError::Internal)?;

        for pepper in self.peppers.iter() {
            let pre = self.prehash(pepper, plain, salt.as_str().as_bytes())?;
            if self.a2.verify_password(&pre, &parsed).is_ok() {
                // здесь можно (в фоне) перехэшировать на «текущий» перец, если совпало старым
                return Ok(true);
            }
        }
        Ok(false)
    }
}
