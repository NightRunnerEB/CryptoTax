use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use tracing::error;

use crate::auth_core::errors::AuthError;

#[derive(Serialize)]
struct ErrBody {
    code: u16,
    message: String,
}

fn map_auth_err(err: AuthError) -> (StatusCode, ErrBody) {
    use AuthError::*;
    let status = match err {
        EmailAlreadyRegistered | PasswordUpdateNotAllowed => StatusCode::CONFLICT,
        UserNotVerified | UserBlocked => StatusCode::FORBIDDEN,
        EmailInvalid | PasswordWeak(_) => StatusCode::UNPROCESSABLE_ENTITY,
        InvalidCredentials | TokenExpired | TokenInvalid | TokenReuse => StatusCode::UNAUTHORIZED,
        RegistrationFailed => StatusCode::BAD_GATEWAY,
        EmailSendFailed => StatusCode::SERVICE_UNAVAILABLE,
        Storage(_) | Internal => StatusCode::INTERNAL_SERVER_ERROR,
    };

    (
        status,
        ErrBody {
            code: status.as_u16(),
            message: err.to_string(),
        },
    )
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        error!(err = ?self, "request failed");
        let (status, error) = map_auth_err(self);
        (status, Json(error)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;

    use super::map_auth_err;
    use crate::auth_core::errors::AuthError;

    #[test]
    fn maps_invalid_credentials_to_unauthorized() {
        let (status, body) = map_auth_err(AuthError::InvalidCredentials);
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(body.code, StatusCode::UNAUTHORIZED.as_u16());
    }

    #[test]
    fn maps_password_weak_to_unprocessable_entity() {
        let (status, body) = map_auth_err(AuthError::PasswordWeak("too_short".to_string()));
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(body.code, StatusCode::UNPROCESSABLE_ENTITY.as_u16());
    }

    #[test]
    fn maps_internal_to_internal_server_error() {
        let (status, body) = map_auth_err(AuthError::Internal);
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(body.code, StatusCode::INTERNAL_SERVER_ERROR.as_u16());
    }
}
