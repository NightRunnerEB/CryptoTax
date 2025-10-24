use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use cache::CacheError;
use serde::Serialize;
use tracing::error;

use crate::domain::error::LedgerError;

#[derive(Serialize)]
struct ErrBody {
    code: u16,
    message: String,
}

fn map_auth_err(err: LedgerError) -> (StatusCode, ErrBody) {
    use LedgerError::*;

    let status = match &err {
        PermissionDenied => StatusCode::FORBIDDEN, // 403
        NotFound {
            ..
        } => StatusCode::NOT_FOUND, // 404

        InvalidTransactionOrder
        | MissingFiatValue {
            ..
        }
        | MissingCostBase {
            ..
        }
        | InvalidFiatValue {
            ..
        }
        | InvalidSwap {
            ..
        }
        | InsufficientBalance {
            ..
        } => StatusCode::UNPROCESSABLE_ENTITY, // 422

        Cache(e) => match e {
            CacheError::Serialization(_) | CacheError::Deserialization(_) | CacheError::Any(_) => StatusCode::INTERNAL_SERVER_ERROR, // 500
            CacheError::Redis(_) | CacheError::RedisConnectionError(_) => StatusCode::SERVICE_UNAVAILABLE,                           // 503
        },

        Db(_) | Internal => StatusCode::INTERNAL_SERVER_ERROR, // 500
    };

    (
        status,
        ErrBody {
            code: status.as_u16(),
            message: err.to_string(),
        },
    )
}

impl IntoResponse for LedgerError {
    fn into_response(self) -> Response {
        error!(err = ?self, "request failed");
        let (status, error) = map_auth_err(self);
        (status, Json(error)).into_response()
    }
}
