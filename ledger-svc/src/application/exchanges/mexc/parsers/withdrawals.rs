use std::collections::HashMap;

use csv_async::StringRecord;
use serde::{Deserialize, Serialize};

use crate::domain::{
    error::Result,
    models::{transaction::Transaction, utils::HeaderView},
    services::{Parser, ParserFactory},
};

/// Funding History > Withdrawal History
#[derive(Deserialize, Serialize)]
pub struct WithdrawalsFactory {
    pub headers: Vec<String>,
}

#[typetag::serde]
impl ParserFactory for WithdrawalsFactory {
    fn id(&self) -> &'static str {
        "mexc.withdrawals"
    }
    fn matches(&self, header: &HeaderView) -> bool {
        header.contains_all(&self.headers)
    }
    fn build(&self, header: &HeaderView) -> Box<dyn Parser> {
        let mut idx = HashMap::new();
        let mut i;
        for name in &self.headers {
            i = header.get(&name).expect("error");
            idx.insert(name.clone(), i);
        }
        Box::new(WithdrawalsParser {
            idx,
        })
    }
}
pub struct WithdrawalsParser {
    idx: HashMap<String, usize>,
}
impl Parser for WithdrawalsParser {
    fn push(&mut self, row: &StringRecord) -> Result<Option<Transaction>> {
        // TODO: Transaction::withdrawal(...)
        let _ = (row, &self.idx);
        Ok(None)
    }
    fn finish(self: Box<Self>) -> Result<Vec<Transaction>> {
        Ok(vec![])
    }
}
