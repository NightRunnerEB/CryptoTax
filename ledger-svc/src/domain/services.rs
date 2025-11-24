use axum::async_trait;
use csv_async::StringRecord;
use futures::io::AsyncRead;

use crate::domain::{
    error::Result,
    models::{
        exchange::ExchangeId,
        transaction::Transaction,
        utils::{HeaderView, ParseContext},
    },
};

pub trait Parser: Send {
    fn push(&mut self, row: &StringRecord) -> Result<Option<Transaction>>;
    fn finish(self: Box<Self>) -> Result<Vec<Transaction>>;
}

#[typetag::serde(tag = "type")]
pub trait ParserFactory: Send + Sync {
    fn id(&self) -> &'static str;
    fn matches(&self, header: &HeaderView) -> bool;
    fn build(&self, header: &HeaderView, ctx: &ParseContext) -> Box<dyn Parser>;
}

#[async_trait]
pub trait ExchangeService: Send + Sync {
    fn id(&self) -> ExchangeId;
    async fn parse_csv(&self, reader: Box<dyn AsyncRead + Send + Unpin>, ctx: ParseContext) -> Result<()>;
    async fn import_api(&self) -> Result<()>;
}
