use serde::{Deserialize, Serialize};

// ---------------------------------------------
//  /coins/list
// ---------------------------------------------
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Coin {
    pub id: String,
    pub symbol: String,
    pub name: String,
    // pub platforms: Option<HashMap<String, Option<String>>>,
}

// ---------------------------------------------
//  /coins/{id}/history
// ---------------------------------------------
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct History {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub localization: Option<Localization>,
    pub market_data: Option<HistoryMarketData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HistoryMarketData {
    pub current_price: CurrentPrice,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Localization {
    pub en: Option<String>,
    pub ru: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CurrentPrice {
    pub usd: Option<f64>,
    pub rub: Option<f64>,
}
