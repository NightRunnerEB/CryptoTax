use std::collections::HashMap;

use csv_async::StringRecord;
use serde::{Deserialize, Serialize};

use crate::domain::{
    error::Result,
    models::{transaction::Transaction, utils::HeaderView},
    services::{Parser, ParserFactory},
};

/// Futures > Futures Capital Flow
#[derive(Deserialize, Serialize)]
pub struct FuturesCapitalFlowFactory {
    pub required_headers: Vec<String>,
}

#[typetag::serde]
impl ParserFactory for FuturesCapitalFlowFactory {
    fn id(&self) -> &'static str {
        "mexc.futures.capital_flow"
    }
    fn matches(&self, header: &HeaderView) -> bool {
        header.contains_all(&self.required_headers)
    }
    fn build(&self, header: &HeaderView) -> Box<dyn Parser> {
        let mut idx = HashMap::new();
        let mut i;
        for name in &self.required_headers {
            i = header.get(&name).expect("error");
            idx.insert(name.clone(), i);
        }
        Box::new(FuturesCapitalFlowParser {
            idx,
        })
    }
}
pub struct FuturesCapitalFlowParser {
    idx: HashMap<String, usize>,
}
impl Parser for FuturesCapitalFlowParser {
    fn push(&mut self, row: &StringRecord) -> Result<Option<Transaction>> {
        // TODO: пополнения/выводы между кошельками фьючерсов, funding, fees
        let _ = (row, &self.idx);
        Ok(None)
    }
    fn finish(self: Box<Self>) -> Result<Vec<Transaction>> {
        Ok(vec![])
    }
}
