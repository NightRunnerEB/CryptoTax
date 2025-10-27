pub mod parsers;

use std::collections::HashMap;

use futures::AsyncRead;

use crate::{
    application::exchanges::ExchangeCfg,
    domain::{
        error::Result,
        models::exchange::ExchangeId,
        ports::TxRepository,
        services::{ExchangeService, ParserFactory},
    },
};

pub struct OkxService<T: TxRepository> {
    tx_repo: T,
    delimiter: char,
    aliases: HashMap<String, String>,
    factories: Vec<Box<dyn ParserFactory>>,
}

impl<T: TxRepository> OkxService<T> {
    pub fn new(tx_repo: T, cfg: ExchangeCfg) -> Self {
        Self {
            tx_repo,
            delimiter: cfg.delimiter,
            aliases: cfg.aliases,
            factories: cfg.factories,
        }
    }
}

#[async_trait::async_trait]
impl<T: TxRepository> ExchangeService for OkxService<T> {
    fn id(&self) -> ExchangeId {
        ExchangeId::Mexc
    }

    async fn parse_csv(&self, reader: Box<dyn AsyncRead + Send + Unpin>) -> Result<()> {
        Ok(())
    }

    async fn import_api(&self) -> Result<()> {
        Ok(())
    }
}
