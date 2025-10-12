use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DerivativeExchangeId {
    pub name: String,
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DerivativeExchangeData {
    pub name: String,
    pub number_of_futures_pairs: i64,
    pub number_of_perpetual_pairs: i64,
    pub tickers: Vec<DerivativeExchangeTicker>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DerivativeExchangeTicker {
    pub symbol: String,
    pub base: String,
    pub target: String,
    pub coin_id: String,
    pub target_coin_id: String,
    pub contract_type: String,
}
