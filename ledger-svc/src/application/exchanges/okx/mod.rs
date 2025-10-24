pub mod parsers;

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
    factories: Vec<Box<dyn ParserFactory>>,
}

impl<T: TxRepository> OkxService<T> {
    pub fn new(tx_repo: T, cfg: ExchangeCfg) -> Self {
        Self {
            tx_repo,
            delimiter: cfg.delimiter,
            factories: cfg.factories,
        }
    }
}

#[async_trait::async_trait]
impl<T: TxRepository> ExchangeService for OkxService<T> {
    fn id(&self) -> ExchangeId {
        ExchangeId::Mexc
    }

    async fn parse_csv(&self) -> Result<()> {
        Ok(())
    }

    async fn import_api(&self) -> Result<()> {
        Ok(())
    }
}
