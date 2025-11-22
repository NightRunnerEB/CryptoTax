use std::collections::HashMap;

use csv_async::StringRecord;
use serde::{Deserialize, Serialize};

use crate::domain::{
    error::Result,
    models::{transaction::Transaction, utils::{HeaderView, ParseContext}},
    services::{Parser, ParserFactory},
};

/// Futures > Futures Trade History
#[derive(Deserialize, Serialize)]
pub struct FuturesTradesFactory {
    pub required_headers: Vec<String>,
}

#[typetag::serde]
impl ParserFactory for FuturesTradesFactory {
    fn id(&self) -> &'static str {
        "mexc.futures.trades"
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
        Box::new(FuturesTradesParser {
            idx,
        })
    }
}
pub struct FuturesTradesParser {
    idx: HashMap<String, usize>,
}
impl Parser for FuturesTradesParser {
    fn push(&mut self, row: &StringRecord) -> Result<Option<Transaction>> {
        // TODO: Transaction::trade_futures(...), учесть side, leverage, fee, pnl
        let _ = (row, &self.idx);
        Ok(None)
    }
    fn finish(self: Box<Self>) -> Result<Vec<Transaction>> {
        Ok(vec![])
    }
}
