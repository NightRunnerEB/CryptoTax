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

fn map_error(err: &LedgerError) -> StatusCode {
    use LedgerError::*;

    match err {
        // 4xx
        PermissionDenied => StatusCode::FORBIDDEN,              // 403
        NotFound(_) => StatusCode::NOT_FOUND,                   // 404
        CsvFormat(_) | Multipart(_) => StatusCode::BAD_REQUEST, // 400

        InvalidTransactionOrder
        | MissingFiatValue(_)
        | MissingCostBase(_)
        | InvalidFiatValue {
            ..
        }
        | InvalidSwap {
            ..
        }
        | InsufficientBalance(_) => StatusCode::UNPROCESSABLE_ENTITY, // 422

        // 5xx
        Cache(e) => match e {
            CacheError::Serialization(_) | CacheError::Deserialization(_) | CacheError::Any(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            } // 500
            CacheError::Redis(_) | CacheError::RedisConnectionError(_) => StatusCode::SERVICE_UNAVAILABLE, // 503
        },

        Db(_) | Rabbitmq(_) | Internal => StatusCode::INTERNAL_SERVER_ERROR, // 500
    }
}

impl IntoResponse for LedgerError {
    fn into_response(self) -> Response {
        let status = map_error(&self);
        error!(err = ?self, status = %status, "request failed");

        let body = ErrBody {
            code: status.as_u16(),
            message: self.to_string(),
        };

        (status, Json(body)).into_response()
    }
}
