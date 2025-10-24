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

// impl FromStr for ExchangeId {
//     type Err = ();
//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         match s.to_lowercase().as_str() {
//             "bybit" => Ok(Self::Bybit),
//             "mexc" => Ok(Self::Mexc),
//             "kucoin" => Ok(Self::Kucoin),
//             "Okx" => Ok(Self::Okx),
//             _ => Err(()),
//         }
//     }
// }

// impl fmt::Display for ExchangeId { /* to_string() для логов/роутов */ }
