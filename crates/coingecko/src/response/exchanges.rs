#![allow(missing_docs)]
use serde::{Deserialize, Serialize};

// ---------------------------------------------
//  /exchanges
// ---------------------------------------------
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Exchange {
    pub id: String,
    pub name: String,
}

// ---------------------------------------------
//  /exchanges/list
// ---------------------------------------------
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExchangeId {
    pub id: String,
    pub name: String,
}

// ---------------------------------------------
//  /exchanges/{id}/tickers
// ---------------------------------------------
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExchangeTickers {
    pub name: String,
    pub tickers: Vec<Ticker>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Ticker {
    pub base: String,
    pub target: String,
    pub coin_id: String,
    pub target_coin_id: Option<String>,
    pub is_stale: bool,
}
