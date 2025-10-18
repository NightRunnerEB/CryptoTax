pub mod utils;

use base64ct::{Base64UrlUnpadded, Encoding};
use chrono::{Duration, Utc};
use rand::RngCore;
use tracing::{error, warn};

use crate::{
    auth_core::{
        errors::AuthError,
        models::*,
        ports::*,
        services::*,
        utils::{normalize_email, validate_password_strength},
    },
    config::VerifyEmailConfig,
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
    pub verify_config: VerifyEmailConfig,
    pub access_ttl: i64,
    pub refresh_ttl: i64,
    pub dummy_password_hash: String,
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
            let (token, hash_bytes, exp) = self.new_email_verification_token();
            self.email_verification.create_token(new_user_id, hash_bytes, &email_norm, exp).await?;

            let link = self.build_verify_link(&token);
            self.mailer
                .send_verification(&email_norm, &link)
                .await
                .map_err(|_| AuthError::EmailSendFailed)?;

            return Ok(());
        };

        let existing: UserWithHash =
            self.users.find_by_email(&email_norm).await?.ok_or_else(|| {
                error!(email = %email_norm, "user exists but not found");
                AuthError::Internal
            })?;

        match existing.status {
            UserStatus::Active | UserStatus::Blocked => Err(AuthError::EmailAlreadyRegistered),
            UserStatus::Pending => {
                self.users.update_password(existing.id, &pwd_hash).await?;
                let (token, hash_bytes, exp) = self.new_email_verification_token();
                if let Err(err) = self.email_verification.revoke_all_for_user(existing.id).await {
                    tracing::error!(
                        user_id = %existing.id,
                        email = %email_norm,
                        ?err,
                        "failed to revoke old email verification tokens"
                    );
                }
                self.email_verification
                    .create_token(existing.id, hash_bytes, &email_norm, exp)
                    .await?;

                let link = self.build_verify_link(&token);
                if let Err(e) = self.mailer.send_verification(&email_norm, &link).await {
                    error!(user_id=%existing.id, email=%email_norm, ?e, "verification email failed after password update");
                    return Err(AuthError::EmailSendFailed);
                }

                Ok(())
            }
        }
    }

    pub async fn verify_email(&self, token_plain: &str) -> Result<(), AuthError> {
        if token_plain.len() > 2048 {
            return Err(AuthError::TokenInvalid);
        }

        let hash = self.refresh_factory.hash(token_plain);

        let Some(user_id) = self.email_verification.consume_by_hash(&hash).await? else {
            return Err(AuthError::TokenInvalid);
        };

        let _ = self.users.activate(user_id).await?;

        Ok(())
    }

    pub async fn login(
        &self,
        email: &str,
        password: &str,
        ip: Option<String>,
        ua: Option<String>,
    ) -> Result<LoginResult, AuthError> {
        let email_norm = normalize_email(email)?;

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
        let access = self.access.issue_token(user.id, session.id, &[])?;
        let pair = self.refresh_factory.new_pair();
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
        let encoded_hash_b64 = Base64UrlUnpadded::encode_string(&hash);

        match self.cache.check_refresh(rec.session_id, &encoded_hash_b64).await {
            Ok(Some(reason)) => match reason {
                RefreshBlockReason::Rotated => {
                    self.handle_reuse(&rec).await?;
                    return Err(AuthError::TokenReuse);
                }
                RefreshBlockReason::RevokedSession => {
                    return Err(AuthError::TokenInvalid);
                }
            },
            Ok(None) => {}
            Err(err) => {
                tracing::warn!(
                    session_id = %rec.session_id,
                    jti        = %rec.jti,
                    ?err,
                    "cache check_refresh failed; fallback to DB"
                );
            }
        }

        if let Some(sess) = self.sessions.get(rec.session_id).await? {
            if !matches!(sess.status, SessionStatus::Active) {
                return Err(AuthError::TokenInvalid);
            }
        } else {
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

        let pair = self.refresh_factory.new_pair();
        let mut new_rec = NewRefresh::from_pair(&pair, rec.user_id, rec.session_id);
        new_rec.parent_jti = Some(rec.jti);
        self.refresh.insert(new_rec).await?;

        if let Err(err) =
            self.cache.mark_refresh_rotated(&encoded_hash_b64, rec.seconds_left()).await
        {
            warn!(session_id=%rec.session_id, jti=%rec.jti, ?err, "redis mark_refresh_rotated failed");
        }
        let _ = self.sessions.touch(rec.session_id).await;

        let access = self.access.issue_token(rec.user_id, rec.session_id, &[])?;
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

        if let Err(err) = self.cache.revoke_all_for_session(sid, self.refresh_ttl).await {
            warn!(session_id=%sid, ?err, "redis revoke_all_for_session failed");
        }

        Ok(())
    }

    pub async fn handle_reuse(&self, rec: &RefreshToken) -> Result<(), AuthError> {
        self.refresh.revoke_all_for_session(rec.session_id).await?;
        if let Err(err) = self.sessions.set_status(rec.session_id, SessionStatus::Revoked).await {
            warn!(session_id=%rec.session_id, ?err, "failed to set session revoked");
        }

        if let Err(err) = self.cache.revoke_all_for_session(rec.session_id, self.refresh_ttl).await
        {
            warn!(session_id=%rec.session_id, ?err, "redis revoke_all_for_session failed");
        }

        Ok(())
    }

    fn new_email_verification_token(&self) -> (String, Vec<u8>, chrono::DateTime<Utc>) {
        let mut raw = [0u8; 32];
        let _ = rand::rngs::OsRng.try_fill_bytes(&mut raw);
        let token = Base64UrlUnpadded::encode_string(&raw);

        let hash_bytes = self.refresh_factory.hash(&token);
        let exp = Utc::now() + Duration::seconds(self.verify_config.token_ttl_secs);
        (token, hash_bytes, exp)
    }

    fn build_verify_link(&self, token: &str) -> String {
        format!("{}{}", self.verify_config.base_url, token)
    }
}
