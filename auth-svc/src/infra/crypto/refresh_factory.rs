use base64ct::{Base64UrlUnpadded, Encoding};
use chrono::{Duration, Utc};
use rand::RngCore;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    auth_core::{models::RefreshPair, services::RefreshTokenFactory},
    config::RefreshConfig,
};

pub struct RefreshFactory {
    pub config: RefreshConfig,
}

impl RefreshFactory {
    pub fn new(config: RefreshConfig) -> Self {
        Self { config }
    }
}

impl RefreshTokenFactory for RefreshFactory {
    fn new_pair(&self) -> RefreshPair {
        let mut raw = [0u8; 32];
        let _ = rand::rngs::OsRng.try_fill_bytes(&mut raw);

        let payload_b64 = Base64UrlUnpadded::encode_string(&raw);
        let plain = format!("{}{}", self.config.prefix, payload_b64);

        let mut h = Sha256::new();
        h.update(plain.as_bytes());
        let token_hash = h.finalize().to_vec();

        let jti = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::seconds(self.config.ttl_secs);

        RefreshPair {
            token_plain: plain,
            token_hash,
            jti,
            expires_at,
        }
    }

    fn hash(&self, token_plain: &str) -> Vec<u8> {
        let mut h = Sha256::new();
        h.update(token_plain.as_bytes());
        h.finalize().to_vec()
    }
}
