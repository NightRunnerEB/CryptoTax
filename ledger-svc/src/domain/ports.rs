use crate::domain::{error::Result, models::transaction::Transaction};

#[async_trait::async_trait]
pub trait TxRepository: Send + Sync {
    async fn insert_batch(&self, rows: &Vec<Transaction>) -> Result<()>;
}
