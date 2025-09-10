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
        EmailAlreadyRegistered => StatusCode::CONFLICT,
        UserNotVerified | UserBlocked => StatusCode::FORBIDDEN,
        EmailInvalid | PasswordWeak(_) => StatusCode::UNPROCESSABLE_ENTITY,
        InvalidCredentials | TokenExpired | TokenInvalid | TokenReuse => StatusCode::UNAUTHORIZED,
        SessionNotFound => StatusCode::NOT_FOUND,
        Storage(_) => StatusCode::SERVICE_UNAVAILABLE,
        EmailSendFailed | Internal => StatusCode::INTERNAL_SERVER_ERROR,
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
