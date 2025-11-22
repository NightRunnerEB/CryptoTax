use std::fmt;

use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum ExchangeId {
    Bybit,
    Mexc,
    Kucoin,
    Okx,
}

impl fmt::Display for ExchangeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ExchangeId::Bybit => "bybit",
            ExchangeId::Mexc => "mexc",
            ExchangeId::Kucoin => "kucoin",
            ExchangeId::Okx => "okx",
        };
        write!(f, "{}", s)
    }
}
