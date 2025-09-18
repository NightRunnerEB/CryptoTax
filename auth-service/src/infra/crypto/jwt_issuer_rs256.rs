use crate::{
    auth_core::{
        errors::AuthError,
        models::{AccessClaims, SignedToken, Uid},
        ports::AccessTokenIssuer,
    },
    config::JwtConfig,
};
use jsonwebtoken::{
    Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode, errors::ErrorKind,
};
use std::collections::HashMap;

#[derive(Clone)]
pub struct JwtKeyRing {
    pub current_kid: String,
    pub enc_keys: HashMap<String, EncodingKey>, // kid -> private (RS256)
    pub dec_keys: HashMap<String, DecodingKey>, // kid -> public (RS256)
}

pub struct JwtIssuerRs {
    pub config: JwtConfig,
    pub keys: JwtKeyRing,
}

impl JwtIssuerRs {
    // НАДО ВЫНЕСТИ ЛОГИКУ ЗАГРУЗКИ КЛЮЧЕЙ В ПРАВИЛЬНОЕ МЕСТО
    pub fn new(config: JwtConfig) -> Self {
        let keys = load_rs_keys();
        Self { config, keys }
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
    fn issue_token(
        &self,
        user_id: Uid,
        session_id: Uid,
        roles: &[String],
    ) -> Result<SignedToken, AuthError> {
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
            roles: roles.to_vec(),
        };
        let header = self.header();
        let enc = self.keys.enc_keys.get(&self.keys.current_kid).ok_or(AuthError::Internal)?;
        let token = encode(&header, &claims, enc).map_err(|_| AuthError::Internal)?;
        Ok(SignedToken { token, exp })
    }

    fn validate(&self, token: &str) -> Result<AccessClaims, AuthError> {
        let header = jsonwebtoken::decode_header(token).map_err(|_| AuthError::TokenInvalid)?;
        let kid = header.kid.ok_or(AuthError::TokenInvalid)?;
        let key = self.keys.dec_keys.get(&kid).ok_or(AuthError::TokenInvalid)?;
        let data =
            decode::<AccessClaims>(token, key, &self.validation()).map_err(|e| match e.kind() {
                ErrorKind::ExpiredSignature => AuthError::TokenExpired,
                _ => AuthError::TokenInvalid,
            })?;
        Ok(data.claims)
    }
}

// TO DO : переписать чтение из .config!
fn load_rs_keys() -> JwtKeyRing {
    use std::fs;
    let kid = "rsa-2025-01".to_string();
    let priv_pem = fs::read("secrets/jwt_rsa_2025_01.pem").unwrap(); // следует из Secrets Manager в проде
    let pub_pem = fs::read("secrets/jwt_rsa_2025_01.pub.pem").unwrap();

    let mut enc_keys = HashMap::new();
    let mut dec_keys = HashMap::new();
    enc_keys.insert(kid.clone(), EncodingKey::from_rsa_pem(&priv_pem).unwrap());
    dec_keys.insert(kid.clone(), DecodingKey::from_rsa_pem(&pub_pem).unwrap());

    JwtKeyRing {
        current_kid: kid,
        enc_keys,
        dec_keys,
    }
}
