use async_trait::async_trait;
use sqlx::{PgPool, Pool, Postgres};

use crate::domain::{error::Result, models::transaction::Transaction, ports::TxRepository};

#[derive(Clone)]
pub struct PgTxRepository {
    inner: Pool<Postgres>,
}

impl PgTxRepository {
    pub fn new(pool: PgPool) -> Self {
        Self {
            inner: pool,
        }
    }
}

#[async_trait]
impl TxRepository for PgTxRepository {
    async fn insert_batch(&self, rows: &Vec<Transaction>) -> Result<()> {
        Ok(())
    }
}
