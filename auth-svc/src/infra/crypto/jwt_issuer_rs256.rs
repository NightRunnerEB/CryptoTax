use std::collections::HashMap;

use base64ct::{Base64UrlUnpadded, Encoding};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode, errors::ErrorKind};
use rsa::{RsaPublicKey, pkcs8::DecodePublicKey, traits::PublicKeyParts};
use serde::Serialize;

use crate::{
    auth_core::{
        errors::AuthError,
        models::{AccessClaims, SignedToken, Uid},
        services::AccessTokenIssuer,
    },
    config::JwtConfig,
};

#[derive(Clone, Debug, Serialize)]
pub struct Jwk {
    pub kty: &'static str,
    pub kid: String,
    #[serde(rename = "use")]
    pub use_: &'static str,
    pub alg: &'static str,
    pub n: String,
    pub e: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct JwksDocument {
    pub keys: Vec<Jwk>,
}

#[derive(Clone)]
pub struct JwtKeyRing {
    pub current_kid: String,
    pub enc_keys: HashMap<String, EncodingKey>, // kid -> private (RS256)
    pub dec_keys: HashMap<String, DecodingKey>, // kid -> public (RS256)
    pub jwks: JwksDocument,
}

pub struct JwtIssuerRs {
    pub config: JwtConfig,
    pub keys: JwtKeyRing,
}

impl JwtIssuerRs {
    // TODO: move key loading behind a real key provider abstraction.
    pub fn new(config: JwtConfig) -> Self {
        let keys = load_rs_keys();
        Self {
            config,
            keys,
        }
    }

    pub fn jwks_document(&self) -> JwksDocument {
        self.keys.jwks.clone()
    }

    fn header(&self) -> Header {
        let mut h = Header::new(Algorithm::RS256);
        h.kid = Some(self.keys.current_kid.clone());
        h
    }

    fn validation(&self) -> Validation {
        let mut v = Validation::new(Algorithm::RS256);
        v.set_issuer(&[self.config.issuer.as_str()]);
        v.set_audience(&[self.config.audience.as_str()]);
        v.leeway = self.config.leeway_secs;
        v.validate_exp = true;
        v.validate_nbf = false;
        v
    }
}

impl AccessTokenIssuer for JwtIssuerRs {
    fn issue_token(&self, user_id: Uid, session_id: Uid, role: &str) -> Result<SignedToken, AuthError> {
        use chrono::Utc;

        let now = Utc::now().timestamp();
        let exp = now + self.config.access_ttl_secs;
        let claims = AccessClaims {
            sub: user_id.to_string(),
            jti: Uid::new_v4().to_string(),
            iss: self.config.issuer.clone(),
            aud: self.config.audience.clone(),
            sid: session_id.to_string(),
            iat: now,
            exp,
            role: role.to_string(),
        };
        let header = self.header();
        let enc = self.keys.enc_keys.get(&self.keys.current_kid).ok_or(AuthError::Internal)?;
        let token = encode(&header, &claims, enc).map_err(|_| AuthError::Internal)?;
        Ok(SignedToken {
            token,
            exp,
        })
    }

    fn validate(&self, token: &str) -> Result<AccessClaims, AuthError> {
        let header = jsonwebtoken::decode_header(token).map_err(|_| AuthError::TokenInvalid)?;
        let kid = header.kid.ok_or(AuthError::TokenInvalid)?;
        let key = self.keys.dec_keys.get(&kid).ok_or(AuthError::TokenInvalid)?;
        let data = decode::<AccessClaims>(token, key, &self.validation()).map_err(|e| match e.kind() {
            ErrorKind::ExpiredSignature => AuthError::TokenExpired,
            _ => AuthError::TokenInvalid,
        })?;
        Ok(data.claims)
    }
}

fn load_rs_keys() -> JwtKeyRing {
    use std::fs;

    let kid = "rsa-2025-01".to_string();
    let priv_pem = fs::read("secrets/jwt_rsa_2025_01.pem").unwrap(); // provided by secret manager in production
    let pub_pem = fs::read("secrets/jwt_rsa_2025_01.pub.pem").unwrap();

    JwtKeyRing::from_pem(kid, &priv_pem, &pub_pem).expect("jwt key ring should load")
}

impl JwtKeyRing {
    fn from_pem(current_kid: String, priv_pem: &[u8], pub_pem: &[u8]) -> Result<Self, AuthError> {
        let mut enc_keys = HashMap::new();
        let mut dec_keys = HashMap::new();

        enc_keys.insert(current_kid.clone(), EncodingKey::from_rsa_pem(priv_pem).map_err(|_| AuthError::Internal)?);
        dec_keys.insert(current_kid.clone(), DecodingKey::from_rsa_pem(pub_pem).map_err(|_| AuthError::Internal)?);

        let jwks = JwksDocument {
            keys: vec![build_rsa_jwk(&current_kid, pub_pem)?],
        };

        Ok(Self {
            current_kid,
            enc_keys,
            dec_keys,
            jwks,
        })
    }
}

fn build_rsa_jwk(kid: &str, pub_pem: &[u8]) -> Result<Jwk, AuthError> {
    let public_key_pem = std::str::from_utf8(pub_pem).map_err(|_| AuthError::Internal)?;
    let public_key = RsaPublicKey::from_public_key_pem(public_key_pem).map_err(|_| AuthError::Internal)?;

    Ok(Jwk {
        kty: "RSA",
        kid: kid.to_string(),
        use_: "sig",
        alg: "RS256",
        n: Base64UrlUnpadded::encode_string(&public_key.n().to_bytes_be()),
        e: Base64UrlUnpadded::encode_string(&public_key.e().to_bytes_be()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const PRIVATE_KEY_PEM: &[u8] = include_bytes!("../../../secrets/jwt_rsa_2025_01.pem");
    const PUBLIC_KEY_PEM: &[u8] = include_bytes!("../../../secrets/jwt_rsa_2025_01.pub.pem");

    fn test_config() -> JwtConfig {
        JwtConfig {
            issuer: "https://auth.cryptotax.local".to_string(),
            audience: "cryptotax.api".to_string(),
            leeway_secs: 5,
            access_ttl_secs: 900,
        }
    }

    fn test_issuer() -> JwtIssuerRs {
        JwtIssuerRs {
            config: test_config(),
            keys: JwtKeyRing::from_pem("rsa-2025-01".to_string(), PRIVATE_KEY_PEM, PUBLIC_KEY_PEM)
                .expect("test key ring should load"),
        }
    }

    #[test]
    fn issue_and_validate_round_trip_keeps_expected_claims() {
        let issuer = test_issuer();
        let user_id = Uid::new_v4();
        let session_id = Uid::new_v4();

        let token = issuer.issue_token(user_id, session_id, "user").expect("token should be issued");
        let header = jsonwebtoken::decode_header(&token.token).expect("header should decode");
        let claims = issuer.validate(&token.token).expect("token should validate");

        assert_eq!(header.alg, Algorithm::RS256);
        assert_eq!(header.kid.as_deref(), Some("rsa-2025-01"));
        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.sid, session_id.to_string());
        assert_eq!(claims.iss, "https://auth.cryptotax.local");
        assert_eq!(claims.aud, "cryptotax.api");
        assert_eq!(claims.role, "user");
    }

    #[test]
    fn jwks_document_contains_current_signing_key() {
        let issuer = test_issuer();
        let jwks = issuer.jwks_document();

        assert_eq!(jwks.keys.len(), 1);

        let jwk = &jwks.keys[0];
        assert_eq!(jwk.kty, "RSA");
        assert_eq!(jwk.kid, "rsa-2025-01");
        assert_eq!(jwk.use_, "sig");
        assert_eq!(jwk.alg, "RS256");
        assert!(!jwk.n.is_empty());
        assert!(!jwk.e.is_empty());
    }
}
