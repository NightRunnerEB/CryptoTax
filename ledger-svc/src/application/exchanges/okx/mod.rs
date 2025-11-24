pub mod parsers;

use std::collections::HashMap;

use axum::async_trait;
use futures::AsyncRead;
use tracing::error;

use crate::{
    application::exchanges::ExchangeCfg,
    domain::{
        error::{LedgerError, Result},
        models::{
            exchange::ExchangeId,
            utils::{HeaderView, ParseContext},
        },
        ports::{ImportCommandRepository, ImportUnitOfWorkFactory},
        services::{ExchangeService, ParserFactory},
    },
};

pub struct OkxService<U, IR>
where
    U: ImportUnitOfWorkFactory + Sync + Send,
    IR: ImportCommandRepository + Sync + Send,
{
    uow_factory: U,
    import_repo: IR,
    delimiter: char,
    aliases: HashMap<String, String>,
    factories: Vec<Box<dyn ParserFactory>>,
}

impl<U, IR> OkxService<U, IR>
where
    U: ImportUnitOfWorkFactory + Sync + Send,
    IR: ImportCommandRepository + Sync + Send,
{
    pub fn new(uow_factory: U, import_repo: IR, cfg: ExchangeCfg) -> Self {
        Self {
            uow_factory,
            import_repo,
            delimiter: cfg.delimiter,
            aliases: cfg.aliases,
            factories: cfg.factories,
        }
    }

    pub fn get_factory(&self, header: &HeaderView) -> Result<&Box<dyn ParserFactory>> {
        self.factories.iter().find(|f| f.matches(header)).ok_or_else(|| {
            let msg = "No matching parser for provided CSV headers";
            error!("{}", msg);
            LedgerError::CsvFormat(msg.to_string())
        })
    }
}

#[async_trait]
impl<U, IR> ExchangeService for OkxService<U, IR>
where
    U: ImportUnitOfWorkFactory + Sync + Send,
    IR: ImportCommandRepository + Sync + Send,
{
    fn id(&self) -> ExchangeId {
        ExchangeId::Okx
    }

    async fn parse_csv(&self, _reader: Box<dyn AsyncRead + Send + Unpin>, _ctx: ParseContext) -> Result<()> {
        Ok(())
    }

    async fn import_api(&self) -> Result<()> {
        Ok(())
    }
}
