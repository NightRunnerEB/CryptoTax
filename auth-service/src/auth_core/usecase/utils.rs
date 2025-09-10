use validator::ValidateEmail;
use zxcvbn::zxcvbn;

use crate::auth_core::errors::AuthError;

const EMAIL_MAX_LEN: usize = 320;
const LOCAL_MAX_LEN: usize = 64;
const DOMAIN_MAX_LEN: usize = 255;

pub fn normalize_email(raw: &str) -> Result<String, AuthError> {
    let email = raw.trim().to_lowercase();

    if email.len() > EMAIL_MAX_LEN {
        return Err(AuthError::EmailInvalid);
    }

    if !email.validate_email() {
        return Err(AuthError::EmailInvalid);
    }
    Ok(email)
}

pub fn validate_password_strength(password: &str, email: &str) -> Result<(), AuthError> {
    // 1) длина
    if password.len() < 10 {
        return Err(AuthError::PasswordWeak("too_short".into()));
    }
    // 2) не содержит email/локальную часть
    let (local, _) = email.split_once('@').unwrap();
    let lower = password.to_lowercase();
    if lower.contains(email) || lower.contains(local) {
        return Err(AuthError::PasswordWeak("contains_email".into()));
    }
    // 3) эвристика zxcvbn (0..=4)
    let estimate = zxcvbn(password, &[email, local])
        .map_err(|_| AuthError::PasswordWeak("estimation_fail".into()))?;
    if estimate.score() < 3 {
        return Err(AuthError::PasswordWeak("zxcvbn_low_score".into()));
    }
    Ok(())
}
