use std::collections::HashMap;

use csv_async::StringRecord;
use serde::{Deserialize, Serialize};

use crate::domain::{
    error::Result,
    models::{
        transaction::Transaction,
        utils::{HeaderView, ParseContext},
    },
    services::{Parser, ParserFactory},
};

/// Futures > Futures Copy Trade Order History
#[derive(Deserialize, Serialize)]
pub struct FuturesCopyTradeOrdersFactory {
    pub required_headers: Vec<String>,
}

#[typetag::serde]
impl ParserFactory for FuturesCopyTradeOrdersFactory {
    fn id(&self) -> &'static str {
        "mexc.futures.copy_trade_orders"
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
        Box::new(FuturesCopyTradeOrdersParser {
            idx,
        })
    }
}
pub struct FuturesCopyTradeOrdersParser {
    idx: HashMap<String, usize>,
}
impl Parser for FuturesCopyTradeOrdersParser {
    fn push(&mut self, row: &StringRecord) -> Result<Option<Transaction>> {
        // TODO: Transaction::futures_copy_trade_order(...)
        let _ = (row, &self.idx);
        Ok(None)
    }
    fn finish(self: Box<Self>) -> Result<Vec<Transaction>> {
        Ok(vec![])
    }
}
