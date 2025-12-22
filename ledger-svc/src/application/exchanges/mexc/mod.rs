pub mod parsers;

use std::collections::HashMap;

use axum::async_trait;
use csv_async::{AsyncReaderBuilder, StringRecord};
use futures::StreamExt;
use futures::io::AsyncRead;
use tracing::error;

use crate::{
    application::exchanges::ExchangeCfg,
    domain::{
        error::{LedgerError, Result},
        models::{
            exchange::ExchangeId,
            import::{Import, ImportStatus},
            transaction::Transaction,
            utils::{HeaderView, ParseContext},
        },
        ports::{ImportCommandRepository, ImportUnitOfWork, ImportUnitOfWorkFactory},
        services::{ExchangeService, ParserFactory},
    },
};

pub struct MexcService<U, IR>
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

impl<U, IR> MexcService<U, IR>
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
impl<U, IR> ExchangeService for MexcService<U, IR>
where
    U: ImportUnitOfWorkFactory + Sync + Send,
    IR: ImportCommandRepository + Sync + Send,
{
    fn id(&self) -> ExchangeId {
        ExchangeId::Mexc
    }

    async fn parse_csv(&self, reader: Box<dyn AsyncRead + Send + Unpin>, ctx: ParseContext) -> Result<()> {
        let mut rdr = AsyncReaderBuilder::new().delimiter(self.delimiter as u8).has_headers(true).create_reader(reader);

        let raw_header = rdr.headers().await.map_err(|e| LedgerError::CsvFormat(e.to_string()))?.clone();
        let header = HeaderView::new(&raw_header, &self.aliases);
        let factory = self.get_factory(&header)?;
        let mut parser = factory.build(&header, &ctx);

        let now = chrono::Utc::now();
        let mut import = Import {
            id: ctx.import_id,
            tenant_id: ctx.tenant_id,
            source: "mexc_csv".to_string(),
            file_name: ctx.file_name.clone(),
            status: ImportStatus::Processing,
            total_count: 0,
            error_summary: None,
            created_at: now,
            completed_at: None,
        };

        self.import_repo.insert_processing(&import).await?;

        let mut uow: Box<dyn ImportUnitOfWork> = self.uow_factory.begin().await?;

        let mut batch: Vec<Transaction> = Vec::with_capacity(100);
        let mut total = 0;
        let mut records = rdr.records();

        while let Some(rec) = records.next().await {
            let rec: StringRecord = match rec {
                Ok(r) => r,
                Err(e) => {
                    let _ = uow.rollback().await;
                    let msg = format!("CSV parse error: {}", e);
                    error!(msg);
                    self.import_repo.mark_failed(import.id, msg.clone()).await?;
                    return Err(LedgerError::CsvFormat(msg));
                }
            };

            if let Some(tx_row) = parser.push(&rec)? {
                batch.push(tx_row);

                if batch.len() >= 100 {
                    uow.transactions().insert_batch(&batch).await?;
                    total += batch.len();
                    batch.clear();
                }
            }
        }

        if !batch.is_empty() {
            uow.transactions().insert_batch(&batch).await?;
            total += batch.len();
            batch.clear();
        }

        let tail = parser.finish()?;
        if !tail.is_empty() {
            uow.transactions().insert_batch(&tail).await?;
            total += tail.len();
        }

        import.status = ImportStatus::Completed;
        import.total_count = total as i32;

        if let Err(e) = uow.mark_import_completed(import.id, import.total_count).await {
            let _ = uow.rollback().await;
            let msg = format!("failed to mark import completed: {e}");
            self.import_repo.mark_failed(import.id, msg.clone()).await?;
            return Err(LedgerError::Db(msg));
        }

        if let Err(e) = uow.outbox().enqueue_transactions_imported(&import).await {
            let _ = uow.rollback().await;
            let msg = format!("failed to enqueue outbox event: {e}");
            self.import_repo.mark_failed(import.id, msg.clone()).await?;
            return Err(LedgerError::Db(msg));
        }

        if let Err(e) = uow.commit().await {
            let msg = format!("failed to commit import UoW: {e}");
            self.import_repo.mark_failed(import.id, msg.clone()).await?;
            return Err(LedgerError::Db(msg));
        }

        Ok(())
    }

    async fn import_api(&self) -> Result<()> {
        Ok(())
    }
}
