use validator::ValidateEmail;
use zxcvbn::zxcvbn;

use crate::auth_core::errors::{AuthError, Result};

const EMAIL_MAX_LEN: usize = 320;

pub fn normalize_email(raw: &str) -> Result<String> {
    let email = raw.trim().to_lowercase();

    if email.len() > EMAIL_MAX_LEN {
        return Err(AuthError::EmailInvalid);
    }

    if !email.validate_email() {
        return Err(AuthError::EmailInvalid);
    }
    Ok(email)
}

pub fn validate_password_strength(password: &str, email: &str) -> Result<()> {
    if password.len() < 10 {
        return Err(AuthError::PasswordWeak("too_short".into()));
    }

    let (local, _) = email.split_once('@').unwrap();
    let lower = password.to_lowercase();
    if lower.contains(email) || lower.contains(local) {
        return Err(AuthError::PasswordWeak("contains_email".into()));
    }

    let estimate = zxcvbn(password, &[email, local]).map_err(|_| AuthError::PasswordWeak("estimation_fail".into()))?;
    if estimate.score() < 3 {
        return Err(AuthError::PasswordWeak("zxcvbn_low_score".into()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth_core::errors::AuthError;

    #[test]
    fn normalize_email_trims_and_lowercases() {
        let email = normalize_email("  USER@Example.COM  ").expect("email should be valid");
        assert_eq!(email, "user@example.com");
    }

    #[test]
    fn normalize_email_rejects_invalid() {
        let err = normalize_email("invalid-email").expect_err("email should be invalid");
        assert!(matches!(err, AuthError::EmailInvalid));
    }

    #[test]
    fn validate_password_strength_rejects_short_password() {
        let err = validate_password_strength("short", "user@example.com").expect_err("password should be rejected");
        assert!(matches!(err, AuthError::PasswordWeak(ref reason) if reason == "too_short"));
    }

    #[test]
    fn validate_password_strength_rejects_email_in_password() {
        let err = validate_password_strength("MyUser@Example.com-password", "user@example.com")
            .expect_err("password should be rejected");
        assert!(matches!(err, AuthError::PasswordWeak(ref reason) if reason == "contains_email"));
    }

    #[test]
    fn validate_password_strength_accepts_strong_password() {
        let res = validate_password_strength("W7!fPq2#Kb9@Lm4$Tx", "user@example.com");
        assert!(res.is_ok(), "strong password should pass validation");
    }
}
