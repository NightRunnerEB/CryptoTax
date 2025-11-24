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

/// Fiat > Fiat OTC
#[derive(Deserialize, Serialize)]
pub struct FiatOtcFactory {
    pub required_headers: Vec<String>,
}

#[typetag::serde]
impl ParserFactory for FiatOtcFactory {
    fn id(&self) -> &'static str {
        "mexc.fiat.otc"
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
        Box::new(FiatOtcParser {
            idx,
        })
    }
}
pub struct FiatOtcParser {
    idx: HashMap<String, usize>,
}
impl Parser for FiatOtcParser {
    fn push(&mut self, row: &StringRecord) -> Result<Option<Transaction>> {
        // TODO: фиатные сделки OTC -> Transaction::fiat_trade(...)
        let _ = (row, &self.idx);
        Ok(None)
    }
    fn finish(self: Box<Self>) -> Result<Vec<Transaction>> {
        Ok(vec![])
    }
}
