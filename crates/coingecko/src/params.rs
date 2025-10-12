use serde::{Deserialize, Serialize};

/// Tickers order for `coin_tickers` and `exchange_tickers`
#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub enum TickersOrder {
    /// Trust Score ascending
    TrustScoreAsc,
    /// Trust Score descending
    TrustScoreDesc,
    /// Base target
    BaseTarget,
}

/// Tickers to include for `derivatives` and `derivatives_exchange`
#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub enum DerivativesIncludeTickers {
    /// All tickers
    All,
    /// Unexpired tickers
    Unexpired,
}
