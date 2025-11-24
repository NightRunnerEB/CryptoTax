use axum::async_trait;
use uuid::Uuid;

use crate::domain::{
    error::Result,
    models::{import::Import, transaction::Transaction},
};

/// Командный репозиторий для imports вне UoW (autocommit).
#[async_trait]
pub trait ImportCommandRepository: Send + Sync {
    async fn insert_processing(&self, import: &Import) -> Result<()>;
    async fn mark_failed(&self, import_id: Uuid, error_summary: String) -> Result<()>;
}

/// Читающий репозиторий для imports (GET в UI / API).
#[async_trait]
pub trait ImportQueryRepository: Send + Sync {
    async fn get(&self, id: Uuid) -> Result<Option<Import>>;
    async fn list_for_tenant(&self, tenant_id: Uuid, limit: i64, offset: i64) -> Result<Vec<Import>>;
}

/// Командный репозиторий транзакций, который работает внутри UoW.
#[async_trait]
pub trait TransactionCommandRepository: Send + Sync {
    async fn insert_batch(&mut self, txs: &[Transaction]) -> Result<()>;
}

/// Читающий репозиторий транзакций.
#[async_trait]
pub trait TransactionQueryRepository: Send + Sync {
    async fn list_by_import(&self, import_id: Uuid) -> Result<Vec<Transaction>>;
    async fn list_for_tenant(&self, tenant_id: Uuid, limit: i64, offset: i64) -> Result<Vec<Transaction>>;
}

/// Outbox для доменного события `transactions.imported`.
#[async_trait]
pub trait OutboxRepository: Send + Sync {
    async fn enqueue_transactions_imported(&mut self, import: &Import) -> Result<()>;
}

/// Unit of Work для сценария импорта.
#[async_trait]
pub trait ImportUnitOfWork: Send {
    fn transactions(&mut self) -> &mut dyn TransactionCommandRepository;
    fn outbox(&mut self) -> &mut dyn OutboxRepository;
    async fn mark_import_completed(&mut self, import_id: Uuid, total_count: i32) -> Result<()>;
    async fn commit(self: Box<Self>) -> Result<()>;
    async fn rollback(self: Box<Self>) -> Result<()>;
}

/// Фабрика юнитов работы.
#[async_trait]
pub trait ImportUnitOfWorkFactory: Send + Sync {
    async fn begin(&self) -> Result<Box<dyn ImportUnitOfWork>>;
}
