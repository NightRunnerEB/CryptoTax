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
    #[error("transactions are out of order")]
    InvalidTransactionOrder,
    #[error("missing USD price for {asset}")]
    MissingFiatValue {
        asset: String,
    },
    #[error("no cost basis found for {asset}")]
    MissingCostBase {
        asset: String,
    },
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
    #[error("insufficient balance: missing {missing}")]
    InsufficientBalance {
        missing: Decimal,
    },
    #[error("not found: {entity}")]
    NotFound {
        entity: &'static str,
    },
    #[error("permission denied")]
    PermissionDenied,
    #[error("internal error")]
    Internal,
}
