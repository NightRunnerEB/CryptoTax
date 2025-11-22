pub mod parsers;

use std::collections::HashMap;

use csv_async::{AsyncReaderBuilder, StringRecord};
use futures::StreamExt;
use futures::io::AsyncRead;
use tracing::error;

use crate::{
    application::exchanges::ExchangeCfg,
    domain::{
        error::{LedgerError, Result},
        models::{exchange::ExchangeId, transaction::Transaction, utils::{HeaderView, ParseContext}},
        ports::TxRepository,
        services::{ExchangeService, ParserFactory},
    },
};

pub struct MexcService<T: TxRepository> {
    tx_repo: T,
    delimiter: char,
    aliases: HashMap<String, String>,
    factories: Vec<Box<dyn ParserFactory>>,
}

impl<T: TxRepository> MexcService<T> {
    pub fn new(tx_repo: T, cfg: ExchangeCfg) -> Self {
        Self {
            tx_repo,
            delimiter: cfg.delimiter,
            aliases: cfg.aliases,
            factories: cfg.factories,
        }
    }

    pub fn get_factory(&self, header: &HeaderView) -> Result<&Box<dyn ParserFactory>> {
        self.factories.iter().find(|f| f.matches(&header)).ok_or_else(|| {
            let msg = "No matching parser for provided CSV headers";
            error!("{}", msg);
            LedgerError::CsvFormat(msg.to_string())
        })
    }
}

#[async_trait::async_trait]
impl<T: TxRepository> ExchangeService for MexcService<T> {
    fn id(&self) -> ExchangeId {
        ExchangeId::Mexc
    }

    async fn parse_csv(&self, reader: Box<dyn AsyncRead + Send + Unpin>, ctx: ParseContext) -> Result<()> {
        let mut rdr = AsyncReaderBuilder::new()
            .delimiter(self.delimiter as u8)
            .has_headers(false)
            .create_reader(reader);

        let raw_header = rdr.headers().await.unwrap().clone();
        let header = HeaderView::new(&raw_header, &self.aliases);
        let factory = self.get_factory(&header)?;
        let mut parser = factory.build(&header, &ctx);

        let mut batch: Vec<Transaction> = Vec::with_capacity(1000);
        let mut records = rdr.records();

        while let Some(rec) = records.next().await {
            let rec: StringRecord = match rec {
                Ok(r) => r,
                Err(e) => {
                    error!("Error reading CSV record: {}", e);
                    return Err(LedgerError::CsvFormat(e.to_string()));
                }
            };

            if let Some(tx) = parser.push(&rec)? {
                batch.push(tx);
                if batch.len() >= 1000 {
                    self.tx_repo.insert_batch(&batch).await?;
                    batch.clear();
                }
            }
        }

        let tail = parser.finish()?;
        if !tail.is_empty() {
            batch.extend(tail);
        }

        if !batch.is_empty() {
            self.tx_repo.insert_batch(&batch).await?;
        }

        Ok(())
    }

    async fn import_api(&self) -> Result<()> {
        Ok(())
    }
}
