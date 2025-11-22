use std::collections::HashMap;

use csv_async::StringRecord;
use serde::{Deserialize, Serialize};

use crate::domain::{
    error::Result,
    models::{transaction::Transaction, utils::{HeaderView, ParseContext}},
    services::{Parser, ParserFactory},
};

/// Spot > Spot Trade History
#[derive(Deserialize, Serialize)]
pub struct SpotTradesFactory {
    pub required_headers: Vec<String>,
}

#[typetag::serde]
impl ParserFactory for SpotTradesFactory {
    fn id(&self) -> &'static str {
        "mexc.spot.trades"
    }
    fn matches(&self, header: &HeaderView) -> bool {
        header.contains_all(&self.required_headers)
    }
    fn build(&self, header: &HeaderView, _ctx: &ParseContext) -> Box<dyn Parser> {
        let mut idx = HashMap::new();
        let mut i;
        for name in &self.required_headers {
            i = header.get(&name).expect("error");
            idx.insert(name.clone(), i);
        }
        Box::new(SpotTradesParser {
            idx,
        })
    }
}
pub struct SpotTradesParser {
    idx: HashMap<String, usize>,
}
impl Parser for SpotTradesParser {
    fn push(&mut self, row: &StringRecord) -> Result<Option<Transaction>> {
        // Пример доступа:
        // let price = row.get(*self.idx.get("Price").ok_or_else(|| anyhow::anyhow!("Price"))?)?;
        // TODO: Transaction::trade_spot(...)
        let _ = (row, &self.idx);
        Ok(None)
    }
    fn finish(self: Box<Self>) -> Result<Vec<Transaction>> {
        Ok(vec![])
    }
}
