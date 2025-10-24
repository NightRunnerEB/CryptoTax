use std::collections::HashMap;

use csv_async::StringRecord;
use serde::{Deserialize, Serialize};

use crate::domain::{
    error::Result,
    models::{transaction::Transaction, utils::HeaderView},
    services::{Parser, ParserFactory},
};

/// Fiat > Fiat P2P Orders
#[derive(Deserialize, Serialize)]
pub struct FiatP2POrdersFactory {
    pub delimiter: String,
    pub required_headers: Vec<String>,
}

#[typetag::serde]
impl ParserFactory for FiatP2POrdersFactory {
    fn id(&self) -> &'static str {
        "mexc.fiat.p2p_orders"
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
        Box::new(FiatP2POrdersParser {
            idx,
        })
    }
}
pub struct FiatP2POrdersParser {
    idx: HashMap<String, usize>,
}
impl Parser for FiatP2POrdersParser {
    fn push(&mut self, row: &StringRecord) -> Result<Option<Transaction>> {
        // TODO: P2P покупка/продажа, комиссии, валюта
        let _ = (row, &self.idx);
        Ok(None)
    }
    fn finish(self: Box<Self>) -> Result<Vec<Transaction>> {
        Ok(vec![])
    }
}
