use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ExchangeAliases {
    pub exchange_id: String,                // e.g. "binance"
    pub generated_at: i64,                  // unix timestamp
    pub coins: HashMap<String, String>,     // name -> cg_id (e.g. "BTC" -> "bitcoin")
}


fn main() {
    println!("Hello, world!");
}
