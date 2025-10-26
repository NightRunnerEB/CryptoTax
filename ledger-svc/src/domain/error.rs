use axum_extra::extract::multipart::MultipartError;
use cache::CacheError;
use rust_decimal::Decimal;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, LedgerError>;

#[derive(Debug, Error)]
pub enum LedgerError {
    #[error(transparent)]
    Cache(#[from] CacheError),
    #[error("Db error: {0}")]
    Db(String),
    #[error("invalid fiat value for {asset}: {value}")]
    InvalidFiatValue {
        asset: String,
        value: Decimal,
    },
    #[error("invalid swap between {from} and {to}")]
    InvalidSwap {
        from: String,
        to: String,
    },
    #[error("transactions are out of order")]
    InvalidTransactionOrder,
    #[error("missing USD price for {0}")]
    MissingFiatValue(String),
    #[error("no cost basis found for {0}")]
    MissingCostBase(String),
    #[error("insufficient balance: missing {0}")]
    InsufficientBalance(Decimal),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("CSV format is incorrect: {0}")]
    CsvFormat(String),
    #[error("permission denied")]
    PermissionDenied,
    #[error("Multipart error: {0}")]
    Multipart(#[from] MultipartError),
    #[error("internal error")]
    Internal,
}
