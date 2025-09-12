pub mod utils;

use base64ct::{Base64UrlUnpadded, Encoding};
use chrono::{Duration, Utc};
use rand::RngCore;
use tracing::warn;

use crate::auth_core::{
    errors::AuthError,
    models::*,
    ports::*,
    utils::{normalize_email, validate_password_strength},
};

pub struct AuthUseCases<
    U: UserRepo,
    S: SessionRepo,
    R: RefreshRepo,
    H: PasswordHasher,
    T: AccessTokenIssuer,
    F: RefreshTokenFactory,
    C: RevocationCache,
    E: EmailVerificationRepo,
    M: Mailer,
> {
    pub users: U,
    pub sessions: S,
    pub refresh: R,
    pub hasher: H,
    pub access: T,
    pub refresh_factory: F,
    pub cache: C,
    pub email_verification: E,
    pub mailer: M,
    pub access_ttl: i64,
    pub refresh_ttl: i64,
    pub verify_email_ttl: i64,
    pub cache_skew: i64,
    pub verify_base_url: String,
    pub dummy_password_hash: String, // для сглаживания времени
}

impl<U, S, R, H, T, F, C, E, M> AuthUseCases<U, S, R, H, T, F, C, E, M>
where
    U: UserRepo,
    S: SessionRepo,
    R: RefreshRepo,
    H: PasswordHasher,
    T: AccessTokenIssuer,
    F: RefreshTokenFactory,
    C: RevocationCache,
    E: EmailVerificationRepo,
    M: Mailer,
{
    pub async fn register(&self, email: &str, password: &str) -> Result<(), AuthError> {
        let email_norm = normalize_email(email)?;
        validate_password_strength(password, &email_norm)?;

        let pwd_hash = self.hasher.hash(password)?;
        if let Some(new_user_id) = self.users.create_user(&email_norm, &pwd_hash).await? {
            let (plain, hash_bytes, exp) = self.new_email_verification_token();
            self.email_verification.create_token(new_user_id, hash_bytes, &email_norm, exp).await?;

            let link = format!("{}{}", self.verify_base_url, plain);
            self.mailer
                .send_verification(&email_norm, &link)
                .await
                .map_err(|_| AuthError::EmailSendFailed)?;

            return Ok(());
        };

        let existing: UserWithHash =
            self.users.find_by_email(&email_norm).await?.ok_or(AuthError::Internal)?; // добавить error!(email=..., "user exists but not found")

        match existing.status {
            UserStatus::Active | UserStatus::Blocked => Err(AuthError::EmailAlreadyRegistered),
            UserStatus::Pending => {
                // Перевыпуск verification
                let (plain, hash_bytes, exp) = self.new_email_verification_token();
                // надо залогировать ошибку
                let _ = self.email_verification.revoke_all_for_user(existing.id).await;
                self.email_verification
                    .create_token(existing.id, hash_bytes, &email_norm, exp)
                    .await?;

                let link = format!("{}{}", self.verify_base_url, plain);
                self.mailer
                    .send_verification(&email_norm, &link)
                    .await
                    .map_err(|_| AuthError::EmailSendFailed)?;

                Ok(())
            }
        }
    }

    pub async fn login(
        &self,
        email: &str,
        password: &str,
        ip: Option<String>,
        ua: Option<String>,
    ) -> Result<LoginResult, AuthError> {
        let email_norm = match normalize_email(email) {
            Ok(v) => v,
            Err(_) => return Err(AuthError::InvalidCredentials),
        };

        let user: UserWithHash = self.users.find_by_email(&email_norm).await?.ok_or_else(|| {
            let _ = self.hasher.verify(&self.dummy_password_hash, password);
            AuthError::InvalidCredentials
        })?;

        if !self.hasher.verify(&user.password_hash, password)? {
            return Err(AuthError::InvalidCredentials);
        }

        match user.status {
            UserStatus::Active => {}
            UserStatus::Pending => return Err(AuthError::UserNotVerified),
            UserStatus::Blocked => return Err(AuthError::UserBlocked),
        }

        let session = self.sessions.create(user.id, ip, ua).await?;
        let access = self.access.issue(user.id, session.id, &[], self.access_ttl)?;
        let pair = self.refresh_factory.new_pair(self.refresh_ttl);
        let rec = NewRefresh::from_pair(&pair, user.id, session.id);
        self.refresh.insert(rec).await?;

        Ok(LoginResult {
            user: user.into_user(),
            session,
            tokens: Tokens {
                access_token: access.token,
                refresh_token: pair.token_plain,
                access_expires_in: self.access_ttl,
                refresh_expires_in: self.refresh_ttl,
            },
        })
    }

    pub async fn refresh(&self, refresh_token: &str) -> Result<Tokens, AuthError> {
        if refresh_token.len() > 2048 {
            return Err(AuthError::TokenInvalid);
        }

        let hash = self.refresh_factory.hash(refresh_token);
        let rec: RefreshToken =
            self.refresh.get_by_hash(&hash).await?.ok_or(AuthError::TokenInvalid)?;

        let hash_b64 = Base64UrlUnpadded::encode_string(&hash);
        if self.cache.is_refresh_blocked(rec.session_id, &hash_b64).await {
            return Err(AuthError::TokenInvalid);
        }

        if let Some(sess) = self.sessions.get(rec.session_id).await? {
            if !matches!(sess.status, SessionStatus::Active) {
                return Err(AuthError::TokenInvalid);
            }
        } else {
            // нет сессии — трактуем как недействительный refresh
            return Err(AuthError::TokenInvalid);
        }

        // Проверки по БД (exp/rotated/revoked)
        match rec.ensure_active() {
            Ok(()) => {}
            Err(e @ AuthError::TokenReuse) => {
                self.handle_reuse(&rec).await?;
                return Err(e);
            }
            Err(e) => return Err(e),
        }

        let rotated = self.refresh.mark_rotated(rec.jti).await?;
        if !rotated {
            self.handle_reuse(&rec).await?;
            return Err(AuthError::TokenReuse);
        }

        let pair = self.refresh_factory.new_pair(self.refresh_ttl);
        let mut new_rec = NewRefresh::from_pair(&pair, rec.user_id, rec.session_id);
        new_rec.parent_jti = Some(rec.jti);
        self.refresh.insert(new_rec).await?;

        let ttl = (rec.seconds_left() + self.cache_skew).max(1);
        if let Err(err) = self.cache.mark_refresh_rotated(&hash_b64, ttl).await {
            warn!(session_id=%rec.session_id, jti=%rec.jti, ?err, "redis mark_refresh_rotated failed");
        }
        let _ = self.sessions.touch(rec.session_id).await;

        let access = self.access.issue(rec.user_id, rec.session_id, &[], self.access_ttl)?;
        Ok(Tokens {
            access_token: access.token,
            refresh_token: pair.token_plain,
            access_expires_in: self.access_ttl,
            refresh_expires_in: self.refresh_ttl,
        })
    }

    pub async fn logout(&self, access: &AccessClaims) -> Result<(), AuthError> {
        let sid = Uid::parse_str(&access.sid).map_err(|_| AuthError::TokenInvalid)?;

        self.refresh.revoke_all_for_session(sid).await?;

        if let Err(err) = self.sessions.set_status(sid, SessionStatus::Closed).await {
            warn!(session_id=%sid, ?err, "failed to set session closed");
        }

        let ttl = (self.refresh_ttl + self.cache_skew).max(1);
        if let Err(err) = self.cache.revoke_all_for_session(sid, ttl).await {
            warn!(session_id=%sid, ?err, "redis revoke_all_for_session failed");
        }

        Ok(())
    }

    pub async fn handle_reuse(&self, rec: &RefreshToken) -> Result<(), AuthError> {
        self.refresh.revoke_all_for_session(rec.session_id).await?;
        if let Err(err) = self.sessions.set_status(rec.session_id, SessionStatus::Revoked).await {
            warn!(session_id=%rec.session_id, ?err, "failed to set session revoked");
        }

        let ttl = (self.refresh_ttl + self.cache_skew).max(1);
        if let Err(err) = self.cache.revoke_all_for_session(rec.session_id, ttl).await {
            warn!(session_id=%rec.session_id, ?err, "redis revoke_all_for_session failed");
        }

        Ok(())
    }

    fn new_email_verification_token(&self) -> (String, Vec<u8>, chrono::DateTime<Utc>) {
        // 32 байта криптослучайных данных → base64url
        let mut raw = [0u8; 32];
        let _ = rand::rngs::OsRng.try_fill_bytes(&mut raw);
        let plain = Base64UrlUnpadded::encode_string(&raw);

        // Храним только хэш
        let hash_bytes = self.refresh_factory.hash(&plain);
        let exp = Utc::now() + Duration::seconds(self.verify_email_ttl);
        (plain, hash_bytes, exp)
    }
}
